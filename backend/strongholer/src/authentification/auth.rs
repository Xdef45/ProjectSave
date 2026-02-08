use sqlx::{mysql, MySqlPool};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, get_current_timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio::{fs, process::Command};
use openssl::aes::{AesKey, unwrap_key, wrap_key};
use argon2::{Argon2, Params};
use std::{env, f32::MIN, result};
use jsonwebtoken::errors::ErrorKind;
use passcheck::PasswordChecker;

// argon2id paramètres
const MEMORY_COST: u32 = 64*1024;
const ITERATION_COST: u32 = 3;
const PARALLELISM_COST: u32 = 4;
const HASH_LENGTH: usize = 32;

//Validiter d'un token Bearer
const EXPIRE_TIME: u64 = 60*20;
const REFRESH_TIME: u64 = 60*10;

const CLIENT_DIRECTORY: &str = "/srv/repos"; 

#[derive(Deserialize)]
pub struct Login{
    pub username: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials{
    pub exp: u64,
    pub id: String,
    pub kdf: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoginState{
    NotSignup,
    KDFError
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogupState{
    KDFError,
    AlreadyExist,
    InvalidPassword,
    /// username inférieur à 5
    UsernameTooShort,
    /// Password length inférieur à 12
    PasswordTooShort,
    /// Pas de caractère spécial
    SpecialCharMissing,
    /// Pas de majuscule
    MajusculeMissing,
    /// Pas de chiffre
    NumberMissing
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BearerState{
    /// Error inconnue
    Error,
    /// Token à expiré
    Expired,
    /// Token à rafraîchir
    Refresh,
    /// Token valide
    Valid
}

#[derive(sqlx::FromRow)]
struct MysqlCredentials{
    id: String,
    encrypt_master_key_2: String
}

#[derive(Clone)]
pub struct Auth{
    db: MySqlPool
}

impl Auth {
    pub async fn new() -> Auth{
        /* Initialisation des paramètre de connection à la base de donnée */
        let opt = mysql::MySqlConnectOptions::new()
        .host(&env::var("DB_HOST").expect("DB_HOST inexistant"))
        .password(&env::var("DB_PASSWORD").expect("DB_PASSWORD inexistant"))
        .port(match env::var("DB_PORT"){
            Ok(result)=> result.parse::<u16>().expect("Conversion du port de str à int a échoué"),
            Err(_)=> Err(()).expect("DB_PORT inexistant")
        })
        .username(&env::var("DB_USER").expect("DB_USER inexistant"))
        .database(&env::var("DB").expect("DB inexistant"));
        Self{db: MySqlPool::connect_with(opt).await.expect("Impossible de se connecter à la DB")}
    }
    pub async fn signup(&self, login: Login) -> Result<String, LogupState> {
        let mut conn = self.db.acquire().await.expect("Impossible d'acquerir une connection DB");
        /* Vérification si l'utilisateur existe */
        let query = sqlx::query("SELECT username FROM Credentials WHERE username=?").bind(login.username.as_str());
        let number_return_line: Vec<mysql::MySqlRow> = query.fetch_all(&mut *conn).await.expect("Une erreur c'est produite");
        if number_return_line.len() > 0 {
            return Err(LogupState::AlreadyExist);
        }

        // Vérification de la validité des Login
        if let Some(validation_state_login) = Auth::validation_login(&login){
            return Err(validation_state_login);
        }

        let username_for_encryption= Auth::corrrect_username_length(&login);
        println!("Username for encryption : {}", username_for_encryption);
        /* Création des id client */
        let kdf_client = match self.create_kdf(&login.password, &username_for_encryption).await {
            Some(kdf_client) => kdf_client,
            None => {
                println!("Erreur lors de la création du kdf");
                return Err(LogupState::KDFError);
            }
        };
        let uuid = Uuid::new_v4().simple().to_string();

        // Création du répertoire utilisateur
        let _ = match Command::new("create_user.sh")
        .args(&[&uuid])
        .output().await{
            Ok(o)=> println!("Erreur : {}\n Sortie : {}", String::from_utf8(o.stderr).expect("msg"), String::from_utf8(o.stdout).expect("msg")),
            Err(_)=> println!("L'installation de la clé ssh client n'a pas fonctionné")
        };

        // Dérivation de la clé
        let path_key = format!("{}/{}/.config/borg/keys/srv_repos_{}_repo", CLIENT_DIRECTORY,uuid, uuid).to_string();
        println!("{}", path_key);
        let master_key = fs::read_to_string(&path_key).await
        .expect("Ouverture du fichier à échoué");
        let _ = fs::remove_file(path_key).await;
        

        let key_encrypted = self.create_master_key_2(&kdf_client, master_key);

        /* Ajout de l'utilisateur dans la base de données */
        let query = sqlx::query("INSERT INTO Credentials (id , username, encrypt_master_key_2) VALUES(?,?,?)")
        .bind(&uuid)
        .bind(login.username.as_str())
        .bind(key_encrypted);
        let _ = query.execute(&mut *conn).await.expect("l'utilisateur n'a pas pu être enregistrer");
        
        /* Renvoyer le cookie JWT */
        let credentials = Credentials{exp: get_current_timestamp() + EXPIRE_TIME, id:uuid, kdf:hex::encode(kdf_client)};
        Ok(self.create_token(&credentials))
    }

    pub fn create_master_key_2(&self, kdf_client:&[u8], master_key: String) -> String{
        /*Chiffrement clé_master_2 */
        if master_key.len() != 553 {
            println!("Master key plus grands que 553{:?}", master_key.len())
        }
        let kdf_key = AesKey::new_encrypt(&kdf_client).expect("wrap kdf n'a pas focntionner");
        let mut in_master_key:[u8; 560]= [0u8; 560];
        in_master_key[..553].copy_from_slice(master_key.as_bytes());
        let mut master_key_2_encrypted: [u8; 568]  = [0u8; 568];
        let _ = wrap_key(&kdf_key, None, &mut master_key_2_encrypted, &in_master_key).expect("Problème lors du chiffrement de la clé master 2");

        /* enregistrer sur un format hexadécimal */
        return hex::encode(&master_key_2_encrypted);
    }

    pub async fn signin(&self, login:Login) -> Result<String, LoginState>{
        /* Récupération clé master 2 */
        let mut conn = self.db.acquire().await.expect("Impossible d'acquerir une connection DB");
        let query = sqlx::query_as("SELECT id, encrypt_master_key_2 FROM Credentials WHERE username=?").bind(login.username.as_str());
        let result: Vec<MysqlCredentials> = query.fetch_all(&mut *conn).await.expect("Une erreur c'est produite");

        /* Vérification si l'utilisateur existe */
        if result.len() != 1 {
            return Err(LoginState::NotSignup);
        }

        /* Création de la clé dériver */
        let username_for_encryption= Auth::corrrect_username_length(&login);
        let kdf_client = match self.create_kdf(&login.password, &username_for_encryption).await {
            Some(kdf_client) => kdf_client,
            None => {
                println!("Erreur lors de la création du kdf");
                return Err(LoginState::KDFError);
            }
        };
        /* Renvoyer le cookie JWT */
        let credentials = Credentials{exp: (get_current_timestamp() + EXPIRE_TIME), id:result[0].id.clone(), kdf:hex::encode(kdf_client)};
        Auth::decrypt_master_2_key_create_file(result[0].encrypt_master_key_2.clone(), &credentials).await;
        /* Vérification du mot de passe */
        Ok(self.create_token(&credentials))
        
    }
    fn validation_login(login: &Login)->Option<LogupState>{
        let checker= PasswordChecker::<'static>::new()
        .min_length(12, Some("12"))  // Use default error message with None OR use Some(str) for use custom message
        .require_upper_lower(Some("A")) // Custom message
        .require_number(Some("1")) // Custom message
        .require_special_char(Some("@")); // Custom message
        if login.username.len() < 3{
            return Some(LogupState::UsernameTooShort);
        }
        match checker.validate(login.password.as_str()) {
            Ok(_)=> return None,
            Err(e)=>{
                match e[0].as_str() {
                    "12" => return Some(LogupState::PasswordTooShort),
                    "A" => return Some(LogupState::MajusculeMissing),
                    "1" => return Some(LogupState::NumberMissing),
                    "@" => return Some(LogupState::SpecialCharMissing),
                    _ => return Some(LogupState::InvalidPassword)
                }
            }
        }
    }

    fn corrrect_username_length(login:&Login)-> String{
        let mut username_for_encryption=String::from(login.username.clone());
        if username_for_encryption.len() < 8{
            let nbr_0_missing = 8 - username_for_encryption.len();
            for _ in 0..nbr_0_missing{
                username_for_encryption.push('0');
            }
            
        }
        return username_for_encryption;
    }

    pub async fn restore_master_key_2(&self, credentials: &Credentials){
        /* Récupération clé master 2 */
        let mut conn = self.db.acquire().await.expect("Impossible d'acquerir une connection DB");
        let query = sqlx::query_as("SELECT id, encrypt_master_key_2 FROM Credentials WHERE id=?").bind(credentials.id.as_str());
        let result: Vec<MysqlCredentials> = query.fetch_all(&mut *conn).await.expect("Une erreur c'est produite");
        Auth::decrypt_master_2_key_create_file(result[0].encrypt_master_key_2.clone(), &credentials).await;

    }
    pub async fn decrypt_master_2_key_create_file(master2_key_encrypted: String, credentials: &Credentials){
        let master2_key_encrypted = hex::decode(master2_key_encrypted).expect("Convertion d'un string en bytes");
        let kdf_key_client = hex::decode(&credentials.kdf).expect("Convertion d'un string en bytes");
        let kdf_key = AesKey::new_decrypt(&kdf_key_client).expect("wrap kdf n'a pas focntionner");
        let mut master_key_2 = [0u8; 560];
        let _ = unwrap_key(&kdf_key, None, &mut master_key_2, &master2_key_encrypted).expect("msg");
        let path = format!("{}/{}/bootstrap/{}.key",CLIENT_DIRECTORY,credentials.id,credentials.id);
        let _ = tokio::fs::write(path, master_key_2).await;
        return
    }

    /* Vérifier token jwt */
    pub fn validation(&self,token_jwt: String)-> (BearerState, (Option<String>, Option<Credentials>)){
        let jwt_secret=env::var("JWT_SECRET").expect("JWT_SECRET inexistant");
        let validation = Validation::new(Algorithm::HS384);
        match decode::<Credentials>(&token_jwt, &DecodingKey::from_secret(jwt_secret.as_bytes()), &validation){
            Err(e)=> {
                if e.into_kind() ==  ErrorKind::ExpiredSignature{
                    return (BearerState::Expired, (None, None))
                }else{

                }
                return (BearerState::Error, (None, None))
            },
            Ok(token)=> {
                if token.claims.exp - REFRESH_TIME > get_current_timestamp(){
                    let credentials =  Credentials { 
                        exp: token.claims.exp, 
                        id: token.claims.id, 
                        kdf: token.claims.kdf };
                    return (BearerState::Valid, (None, Some(credentials)))
                    
                }else{
                    let credentials =  Credentials { 
                        exp: get_current_timestamp()+EXPIRE_TIME, 
                        id: token.claims.id, 
                        kdf: token.claims.kdf };
                    return (BearerState::Refresh, (Some(self.create_token(&credentials)), Some(credentials)))
                }
            }
        };
    }

    pub fn decode_token(token_jwt: &String) -> Credentials{
        let jwt_secret=env::var("JWT_SECRET").expect("JWT_SECRET inexistant");
        let validation = Validation::new(Algorithm::HS384);
        let token = decode::<Credentials>(
                &token_jwt,
                &DecodingKey::from_secret(jwt_secret.as_bytes()),
                &validation
            ).expect("le decodage du token c'est mal déroulé");
        let credentials =  Credentials { 
            exp: token.claims.exp, 
            id: token.claims.id, 
            kdf: token.claims.kdf };
        return credentials
    }

    fn create_token(&self, credentials: &Credentials) -> String{
        /* Récupère la variable d'environnement */
        let jwt_secret=env::var("JWT_SECRET").expect("JWT_SECRET inexistant");

        let header = Header::new(Algorithm::HS384);
        let token = match encode(&header, &credentials, &EncodingKey::from_secret(jwt_secret.as_bytes())){
            Ok(token) => token,
            Err(_)=> "erreur".to_string()
        };
        token
    }

    async fn create_kdf(&self, password: &String, salt: &String) -> Option<[u8; HASH_LENGTH]>{
        let output_len: Option<usize> = Some(HASH_LENGTH);
        let param: Params= argon2::Params::new(MEMORY_COST, ITERATION_COST, PARALLELISM_COST, output_len).expect("problème");
        let password = password.as_bytes();
        let salt = salt.as_bytes();
        let mut out= [0u8; HASH_LENGTH];
        let result = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, param).hash_password_into(password, salt, &mut out);
        match result {
            Ok(_)=> return Some(out),
            Err(e)=>{
                println!("Erreur lors de la création du kdf : {:?}", e);
                return None
            }
        }
    }
}
