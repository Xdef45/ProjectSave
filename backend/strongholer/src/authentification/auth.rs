use sqlx::{mysql, Connection};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, get_current_timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use openssl::rand::rand_bytes;
use openssl::aes::{AesKey, unwrap_key, wrap_key};
use argon2::{Argon2, Params};
use config;


const MEMORY_COST: u32 = 64*1024;
const ITERATION_COST: u32 = 3;
const PARALLELISM_COST: u32 = 4;
const HASH_LENGTH: usize = 32;



#[derive(Debug, Deserialize)]
struct DbSettings {
    db_host: String,
    db_port: u16,
    db_user: String,
    db_password: String,
    db: String,
    jwt_secret: String
}


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
#[derive(Debug, Serialize, Deserialize)]
pub enum LoginState{
    AlreadyExist,
    NotSignup,
    InvalidPassword
}
#[derive(sqlx::FromRow)]
struct MysqlCredentials{
    id: String,
    encrypt_master_key_2: String
}

pub struct Auth;

impl Auth {
    async fn db(&mut self) -> sqlx::MySqlConnection{
        let db_setting: DbSettings = config::Config::builder()
        .add_source(config::File::with_name(".env.json"))
        .build()
        .expect("La lecture du fichier .env a échoué").try_deserialize().expect("La déserialisation aéchoué");
        let opt = mysql::MySqlConnectOptions::new()
        .host(db_setting.db_host.as_str())
        .password(db_setting.db_password.as_str())
        .port(db_setting.db_port)
        .username(db_setting.db_user.as_str())
        .database(db_setting.db.as_str());
        return mysql::MySqlConnection::connect_with(&opt).await.unwrap();
    }
    pub async fn signup(&mut self, login: Login) -> Result<String, LoginState> {
        /* Initialisation des paramètre de connection à la base de donnée */
        let mut connection = self.db().await;

        /* Vérification si l'utilisateur existe */
        let query = sqlx::query("SELECT username FROM Credentials WHERE username=?").bind(login.username.as_str());
        let number_return_line: Vec<mysql::MySqlRow> = query.fetch_all(&mut connection).await.expect("Une erreur c'est produite");
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
        let _ = query.execute(&mut connection).await.expect("l'utilisateur n'a pas pu être enregistrer");
        
        /* Renvoyer le cookie JWT */
        let credentials = Credentials{exp: get_current_timestamp(), id:uuid, kdf:hex::encode(kdf_client)};
        Ok(self.create_token(credentials).await)
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

    pub async fn signin(&mut self, login:Login) -> Result<String, LoginState>{
        /* Initialisation des paramètre de connection à la base de donnée */
        let mut connection = self.db().await;

        /* Récupération clé master 2 */
        let query = sqlx::query_as("SELECT id, encrypt_master_key_2 FROM Credentials WHERE username=?").bind(login.username.as_str());
        let result: Vec<MysqlCredentials> = query.fetch_all(&mut connection).await.expect("Une erreur c'est produite");

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
        let credentials = Credentials{exp: get_current_timestamp(), id:result[0].id.clone(), kdf:hex::encode(kdf_client)};
        Ok(self.create_token(credentials).await)
    }

    /* Vérifier token jwt */
    pub async fn validation(self,token_jwt: String)-> Result<Credentials, String>{
        let jwt_token: DbSettings = config::Config::builder()
        .add_source(config::File::with_name(".env.json"))
        .build()
        .expect("La lecture du fichier .env a échoué").try_deserialize().expect("La déserialisation aéchoué");
        let mut validation = Validation::new(Algorithm::HS384);
        validation.leeway=60*10;
        match decode::<Credentials>(&token_jwt, &DecodingKey::from_secret(jwt_token.jwt_secret.as_bytes()), &validation){
            Err(_)=> return Err("Cookie expirer".to_string()),
            Ok(o)=> return Ok(o.claims)
        };

    }

    async fn create_token(&self, credentials: Credentials) -> String{
        /* Récupère la variable d'environnement */
        let jwt_token: DbSettings = config::Config::builder()
        .add_source(config::File::with_name(".env.json"))
        .build()
        .expect("La lecture du fichier .env a échoué").try_deserialize().expect("La déserialisation aéchoué");

        let header = Header::new(Algorithm::HS384);
        let token = match encode(&header, &credentials, &EncodingKey::from_secret(jwt_token.jwt_secret.as_bytes())){
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
