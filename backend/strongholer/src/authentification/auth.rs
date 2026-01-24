use sqlx::{mysql, MySqlPool};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, get_current_timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use openssl::rand::rand_bytes;
use openssl::aes::{AesKey, unwrap_key, wrap_key};
use argon2::{Argon2, Params};
use std::env;
use jsonwebtoken::errors::ErrorKind;

// argon2id paramètres
const MEMORY_COST: u32 = 64*1024;
const ITERATION_COST: u32 = 3;
const PARALLELISM_COST: u32 = 4;
const HASH_LENGTH: usize = 32;

//Validiter d'un token Bearer
const EXPIRE_TIME: u64 = 60*20;
const REFRESH_TIME: u64 = 60*10;

#[derive(Deserialize)]
pub struct Login{
    pub username: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials{
    exp: u64,
    id: String,
    kdf: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoginState{
    AlreadyExist,
    NotSignup,
    InvalidPassword
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
    pub async fn signup(&self, login: Login) -> Result<String, LoginState> {
        let mut conn = self.db.acquire().await.expect("Impossible d'acquerir une connection DB");
        /* Vérification si l'utilisateur existe */
        let query = sqlx::query("SELECT username FROM Credentials WHERE username=?").bind(login.username.as_str());
        let number_return_line: Vec<mysql::MySqlRow> = query.fetch_all(&mut *conn).await.expect("Une erreur c'est produite");
        if number_return_line.len() > 0 {
            return Err(LoginState::AlreadyExist);
        }

        /* Ajout de l'utilisateur à la base de donnée */
        let kdf_client:[u8; 32]  = self.create_kdf(&login.password, &login.username).await;
        let uuid = Uuid::new_v4().hyphenated().to_string();

        let key_encrypted = self.create_master_key_2(&kdf_client);

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

    pub fn create_master_key_2(&self, kdf_client:&[u8]) -> String{
        /* Création clé_master */
        let mut master_key = [0u8;32];
        rand_bytes(&mut master_key).expect("La clé master n'a pas pu être créer correctement");


        /*Chiffrement clé_master_2 */
        let kdf_key = AesKey::new_encrypt(&kdf_client).expect("wrap kdf n'a pas focntionner");
        let mut master_key_2_encrypted = [0u8; 40];
        let _ = wrap_key(&kdf_key, None, &mut master_key_2_encrypted, &master_key).expect("Problème lors du chiffrement de la clé master 2");

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
        let kdf_client:[u8; 32]  = self.create_kdf(&login.password, &login.username).await;

        /* Convertion hex to bytes */
        let master_key_2_encrypted = hex::decode(result[0].encrypt_master_key_2.clone()).expect("Problème lors de la convertion hex to byte");

        /* Vérification du mot de passe */
        let kdf_key = AesKey::new_decrypt(&kdf_client).expect("wrap kdf n'a pas focntionner");
        let mut master_key_2 = [0u8; 32];
        let _ = match unwrap_key(&kdf_key, None, &mut master_key_2, &master_key_2_encrypted){
            Err(_)=> return Err(LoginState::InvalidPassword),
            Ok(o)=>o
        };

        /* Renvoyer le cookie JWT */
        let credentials = Credentials{exp: (get_current_timestamp() + EXPIRE_TIME), id:result[0].id.clone(), kdf:hex::encode(kdf_client)};
        Ok(self.create_token(&credentials))
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

    async fn create_kdf(&self, password: &String, salt: &String) -> [u8; HASH_LENGTH]{
        let output_len: Option<usize> = Some(HASH_LENGTH);
        let param: Params= argon2::Params::new(MEMORY_COST, ITERATION_COST, PARALLELISM_COST, output_len).expect("problème");
        let password = password.as_bytes();
        let salt = salt.as_bytes();
        let mut out= [0u8; HASH_LENGTH];
        let _ = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, param).hash_password_into(password, salt, &mut out);
        return out;
    }
}
