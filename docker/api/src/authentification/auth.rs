use sqlx::{mysql, MySqlPool};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, get_current_timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use openssl::aes::{AesKey, unwrap_key, wrap_key};
use argon2::{Argon2, Params};
use std::env;
use jsonwebtoken::errors::ErrorKind;
use passcheck::PasswordChecker;
use openssh::{Session, KnownHosts};
use std::sync::Arc;
use openssh_sftp_client::{Sftp, SftpOptions};
use crate::{borg_script::create_user, error::APIError};

// argon2id paramètres
const MEMORY_COST: u32 = 64*1024;
const ITERATION_COST: u32 = 3;
const PARALLELISM_COST: u32 = 4;
const HASH_LENGTH: usize = 32;

//Validiter d'un token Bearer
const EXPIRE_TIME: u64 = 60*60;
const REFRESH_TIME: u64 = 60*30;

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
pub enum BearerState{
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
    db: MySqlPool,
    pub ssh_connexion: Arc<Session>,
    pub sftp_connexion: Arc<Sftp>
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
        let session_ssh = Session::connect_mux("ssh://borg", KnownHosts::Add).await.expect("Impossible de se connecter au serveur ssh");
        let session_sftp = Session::connect_mux("ssh://borg", KnownHosts::Add).await.expect("Impossible de se connecter au serveur ssh");
        Self{
            db: MySqlPool::connect_with(opt).await.expect("Impossible de se connecter à la DB"),
            ssh_connexion: Arc::new(session_ssh),
            sftp_connexion: Arc::new(Sftp::from_session(session_sftp, SftpOptions::default()).await.expect("test"))
        }
    }
    pub async fn signup(&self, login: Login) -> Result<String, APIError> {
        let mut conn = self.db.acquire().await.expect("Impossible d'acquerir une connection DB");
        /* Vérification si l'utilisateur existe */
        let query = sqlx::query("SELECT username FROM Credentials WHERE username=?").bind(login.username.as_str());
        let number_return_line: Vec<mysql::MySqlRow> = query.fetch_all(&mut *conn).await.expect("Une erreur c'est produite");
        if number_return_line.len() > 0 {
            return Err(APIError::AlreadyExist);
        }

        // Vérification de la validité des Login
        if let Some(validation_state_login) = Auth::validation_login(&login){
            return Err(validation_state_login);
        }

        let username_for_encryption= Auth::corrrect_username_length(&login);
        println!("Username for encryption : {}", username_for_encryption);
        /* Création des id client */
        let kdf_client = match self.create_kdf(&login.password, &username_for_encryption).await {
            Ok(kdf_client) => kdf_client,
            Err(e) => {
                println!("Erreur lors de la création du kdf");
                return Err(e);
            }
        };
        let uuid = Uuid::new_v4().simple().to_string();

        // Création du répertoire utilisateur
        let master_key = match create_user::create_user(&uuid, self.ssh_connexion.clone(), self.sftp_connexion.clone()).await{
            Ok(key)=>key,
            Err(e)=>return Err(e)
        };

        // Dérivation de la clé
        let key_encrypted = match self.create_master_key_2(&kdf_client, master_key){
            Ok(key)=>key,
            Err(e)=>return Err(e)
        };

        // Supression de la clé
        let _ = match self.delete_master_key_2_file(&uuid).await{
            Ok(_)=>(),
            Err(e)=>return Err(e)
        };

        /* Ajout de l'utilisateur dans la base de données */
        let query = sqlx::query("INSERT INTO Credentials (id , username, encrypt_master_key_2) VALUES(?,?,?)")
        .bind(&uuid)
        .bind(login.username.as_str())
        .bind(key_encrypted);
        let _ = query.execute(&mut *conn).await.expect("l'utilisateur n'a pas pu être enregistrer");
        
        /* Renvoyer le cookie JWT */
        let credentials = Credentials{exp: get_current_timestamp() + EXPIRE_TIME, id:uuid, kdf:hex::encode(kdf_client)};
        match self.create_token(&credentials){
            Ok(token)=>return Ok(token),
            Err(e)=>return Err(e)
        };
    }

    pub fn create_master_key_2(&self, kdf_client:&[u8], master_key: String) -> Result<String, APIError>{
        /*Chiffrement clé_master_2 */
        if master_key.len() != 553 {
            println!("Master key plus grands que 553{:?}", master_key.len());
            return Err(APIError::KDFError);
        }
        let kdf_key = AesKey::new_encrypt(&kdf_client).expect("wrap kdf n'a pas focntionner");
        let mut in_master_key:[u8; 560]= [0u8; 560];
        in_master_key[..553].copy_from_slice(master_key.as_bytes());
        let mut master_key_2_encrypted: [u8; 568]  = [0u8; 568];
        let Ok(_) = wrap_key(&kdf_key, None, &mut master_key_2_encrypted, &in_master_key)else{
            println!("Erreur lors de la création de la clé KDF");
            return Err(APIError::KDFError)
        };

        /* enregistrer sur un format hexadécimal */
        return Ok(hex::encode(&master_key_2_encrypted));
    }

    pub async fn signin(&self, login:Login) -> Result<String, APIError>{
        /* Récupération clé master 2 */
        let mut conn = self.db.acquire().await.expect("Impossible d'acquerir une connection DB");
        let query = sqlx::query_as("SELECT id, encrypt_master_key_2 FROM Credentials WHERE username=?").bind(login.username.as_str());
        let result: Vec<MysqlCredentials> = query.fetch_all(&mut *conn).await.expect("Une erreur c'est produite");

        /* Vérification si l'utilisateur existe */
        if result.len() != 1 {
            return Err(APIError::NotSignup);
        }

        /* Création de la clé dériver */
        let username_for_encryption= Auth::corrrect_username_length(&login);
        let kdf_client = match self.create_kdf(&login.password, &username_for_encryption).await {
            Ok(kdf_client) => kdf_client,
            Err(e) => {
                return Err(e);
            }
        };
        /* Renvoyer le cookie JWT */
        let credentials = Credentials{exp: (get_current_timestamp() + EXPIRE_TIME), id:result[0].id.clone(), kdf:hex::encode(kdf_client)};
        let _ = match self.decrypt_master_2_key(&credentials).await{
            Ok(_)=>(),
            Err(e)=>return Err(e)
        };
        /* Vérification du mot de passe */
        match self.create_token(&credentials){
            Ok(token)=>return Ok(token),
            Err(e)=>return Err(e)
        };
        
    }
    fn validation_login(login: &Login)->Option<APIError>{
        let checker= PasswordChecker::<'static>::new()
        .min_length(12, Some("12"))  // Use default error message with None OR use Some(str) for use custom message
        .require_upper_lower(Some("A")) // Custom message
        .require_number(Some("1")) // Custom message
        .require_special_char(Some("@")); // Custom message
        if login.username.len() < 3{
            return Some(APIError::UsernameTooShort);
        }
        match checker.validate(login.password.as_str()) {
            Ok(_)=> return None,
            Err(e)=>{
                match e[0].as_str() {
                    "12" => return Some(APIError::PasswordTooShort),
                    "A" => return Some(APIError::MajusculeMissing),
                    "1" => return Some(APIError::NumberMissing),
                    "@" => return Some(APIError::SpecialCharMissing),
                    _ => return Some(APIError::InvalidPassword)
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

    pub async fn restore_master_key_2_file(&self, credentials: &Credentials)-> Result<(),APIError>{
        let filename = format!("{}/{}/.config/borg/keys/srv_repos_{}_repo", CLIENT_DIRECTORY, credentials.id, credentials.id);

        //Vérification de la présence de la clé
        let output = match self.ssh_connexion.command("test").args(["-f", filename.as_str()]).output().await{
            Ok(o)=>o,
            Err(_)=>{println!("Erreur connexion ssh");return Err(APIError::Ssh)}
        };
        let stdout = match String::from_utf8(output.stdout.clone()){
            Ok(out)=>out,
            Err(_)=>{
                println!("Erreur conversion stdout UTF8 decrypt_master_2_key_create");
                return Err(APIError::UTF8)
            }
        };
        let stderr = match String::from_utf8(output.stderr.clone()){
            Ok(out)=>out,
            Err(_)=>{
                println!("Erreur conversion stderr UTF8 decrypt_master_2_key_create");
                return Err(APIError::UTF8)
            }
        };
        if output.status.success(){
            println!("Erreur la vérification présence de la clé master key already existe decrypt_master_2_create_file : \nStdout: {}\nErreur: {}", stdout, stderr);
            return Ok(())//Err(APIError::Script)
        }
        // Déchiffrement de la clé Borg
        let master_key_2 = match self.decrypt_master_2_key(&credentials).await{
            Ok(key)=>key,
            Err(e)=>return Err(e)
        };
        // Création du fichier de la clé Borg
        let mut key_borg = match self.sftp_connexion.create(&filename).await {
            Ok(f)=>f,
            Err(e)=>{
            println!("Ouverture du fichier pour écrire la clé borg: {}", e.to_string());
            return Err(APIError::Sftp);
            }
        };
        match key_borg.write_all(&master_key_2).await{
            Ok(_)=>return Ok(()),
            Err(_)=> return Err(APIError::Write)
        };
    }

    async fn decrypt_master_2_key(&self, credentials: &Credentials)-> Result<[u8; 560],APIError>{  
        /* Récupération clé master 2 */
        let mut conn = self.db.acquire().await.expect("Impossible d'acquerir une connection DB");
        let query = sqlx::query_as("SELECT id, encrypt_master_key_2 FROM Credentials WHERE id=?").bind(credentials.id.as_str());
        let result: Vec<MysqlCredentials> = query.fetch_all(&mut *conn).await.expect("Une erreur c'est produite");
        if result.len() != 1 {
            println!("L'utilisateur {} n'est pas connue dans la base de données", credentials.id);
            return Err(APIError::NotSignup);
        }
        // Déchiffrement de la clé
        let master2_key_encrypted = hex::decode(result[0].encrypt_master_key_2.clone()).expect("Convertion d'un string en bytes");
        let kdf_key_client = hex::decode(&credentials.kdf).expect("Convertion d'un string en bytes");
        let kdf_key = AesKey::new_decrypt(&kdf_key_client).expect("wrap kdf n'a pas focntionner");
        let mut master_key_2 = [0u8; 560];
        let Ok(_) = unwrap_key(&kdf_key, None, &mut master_key_2, &master2_key_encrypted)else {
            println!("Erreur déchiffrement de la clé master 2");
            return Err(APIError::KDFError)
        };
        Ok(master_key_2)
    }

    pub async fn delete_master_key_2_file(&self, uuid: &String)->Result<(), APIError>{
        let filename = format!("{}/{}/.config/borg/keys/srv_repos_{}_repo", CLIENT_DIRECTORY, uuid, uuid);
        let output = match self.ssh_connexion.command("rm").arg(filename).output().await{
            Ok(o)=>o,
            Err(_)=>{
                println!("Erreur connexion ssh supression clé borg delete_master_key_2_file");
                return Err(APIError::Ssh)
            }
        };
        let stdout = match String::from_utf8(output.stdout.clone()){
            Ok(out)=>out,
            Err(_)=>{
                println!("Erreur conversion stdout UTF8 delete_master_2_key_file");
                return Err(APIError::UTF8)
            }
        };
        let stderr = match String::from_utf8(output.stderr.clone()){
            Ok(out)=>out,
            Err(_)=>{
                println!("Erreur conversion stderr UTF8 delete_master_2_key_file");
                return Err(APIError::UTF8)
            }
        };
        if ! output.status.success(){
            println!("Erreur lors de la supression de la clé borg : \nStdout: {}\nErreur: {}", stdout, stderr);
        }
        return Ok(())
    }

        pub fn decode_token(token_jwt: &str) -> Result<Credentials, APIError>{
        let jwt_secret=env::var("JWT_SECRET").expect("JWT_SECRET inexistant");
        let validation = Validation::new(Algorithm::HS384);
        let token = match decode::<Credentials>(
                token_jwt,
                &DecodingKey::from_secret(jwt_secret.as_bytes()),
                &validation
            ){
                Ok(t)=>t,
                Err(e)=>{
                    println!("le decodage du token c'est mal déroulé");
                    if e.into_kind() ==  ErrorKind::ExpiredSignature{
                        println!("Le cookie Bearer a expiré");
                        return Err(APIError::Expired)
                    }else{
                        println!("Erreur inconnue concernant la validation du cookie");
                        return Err(APIError::ErrorBearer)
                    }
                }
            };
        let credentials =  Credentials { 
            exp: token.claims.exp, 
            id: token.claims.id, 
            kdf: token.claims.kdf };
        return Ok(credentials)
    }

    /* Vérifier token jwt */
    pub fn validation(&self,token_jwt: String)-> Result<(BearerState, (Option<String>, Credentials)), APIError>{
        let mut credentials = Auth::decode_token(token_jwt.as_str())?;
        if credentials.exp - REFRESH_TIME > get_current_timestamp(){
            return Ok((BearerState::Valid, (None, credentials)))
        }else{
            credentials.exp = get_current_timestamp()+EXPIRE_TIME;
            let token = self.create_token(&credentials)?;
            return Ok((BearerState::Refresh, (Some(token), credentials)))
        }
    }

    fn create_token(&self, credentials: &Credentials) -> Result<String, APIError>{
        /* Récupère la variable d'environnement */
        let jwt_secret=env::var("JWT_SECRET").expect("JWT_SECRET inexistant");

        let header = Header::new(Algorithm::HS384);
        match encode(&header, &credentials, &EncodingKey::from_secret(jwt_secret.as_bytes())){
            Ok(token) => return Ok(token),
            Err(e)=> {
                println!("Erreu lors de l'encodage du token create_token {}", e.to_string());
                return Err(APIError::EncodeToken)
            }
        };
    }

    async fn create_kdf(&self, password: &String, salt: &String) -> Result<[u8; HASH_LENGTH], APIError>{
        let output_len: Option<usize> = Some(HASH_LENGTH);
        let param: Params= argon2::Params::new(MEMORY_COST, ITERATION_COST, PARALLELISM_COST, output_len).expect("problème");
        let password = password.as_bytes();
        let salt = salt.as_bytes();
        let mut out= [0u8; HASH_LENGTH];
        let result = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, param).hash_password_into(password, salt, &mut out);
        match result {
            Ok(_)=> return Ok(out),
            Err(e)=>{
                println!("Erreur lors de la création du kdf : {:?}", e);
                return Err(APIError::KDFError)
            }
        }
    }
}
