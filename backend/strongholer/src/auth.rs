use actix_web::http::header::RETRY_AFTER;
use sqlx::{mysql, Connection};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use crate::kdfpassword::create_kdf;
use uuid::Uuid;
use openssl::rand::rand_bytes;
use openssl::aes::{AesKey, unwrap_key, wrap_key};
use openssl::sha::sha256;
use subtle::ConstantTimeEq;

#[derive(Deserialize)]
pub struct Login{
    pub username: String,
    pub password: String
}
#[derive(Debug, Serialize, Deserialize)]
struct Credentials{
    id: String,
    kdf: String
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
    pub async fn signup(&mut self, login: Login) -> Result<String, LoginState> {
        /* Initialisation des paramètre de connection à la base de donnée */
        let opt = mysql::MySqlConnectOptions::new().host("127.0.0.1").password("mypass").port(3306).username("root").database("strongholder");
        let mut connection = mysql::MySqlConnection::connect_with(&opt).await.unwrap();

        /* Vérification si l'utilisateur existe */
        let query = sqlx::query("SELECT username FROM Credentials WHERE username=?").bind(login.username.as_str());
        let number_return_line: Vec<mysql::MySqlRow> = query.fetch_all(&mut connection).await.expect("Une erreur c'est produite");
        if number_return_line.len() > 0 {
            return Err(LoginState::AlreadyExist);
        }

        /* Ajout de l'utilisateur à la base de donnée */
        let kdf_client:[u8; 32]  = create_kdf(&login.password, &login.username);
        let uuid = Uuid::new_v4().hyphenated().to_string();

        let key_encrypted = self.create_master_key_2(&kdf_client);

        /* Ajout de l'utilisateur dans la base de données */
        let query = sqlx::query("INSERT INTO Credentials (id , username, encrypt_master_key_2) VALUES(?,?,?)")
        .bind(&uuid)
        .bind(login.username.as_str())
        .bind(key_encrypted);
        let _ = query.execute(&mut connection).await.expect("l'utilisateur n'a pas pu être enregistrer");
        
        /* Renvoyer le cookie JWT */
        let credentials = Credentials{id:uuid, kdf:hex::encode(kdf_client)};
        Ok(self.create_token(credentials))
    }

    pub fn create_master_key_2(&self, kdf_client:&[u8]) -> String{
        /* Création clé_master */
        let mut master_key = [0u8;32];
        rand_bytes(&mut master_key).expect("La clé master n'a pas pu être créer correctement");

        /* Création du hash de laclé master */
        let hash_master_key: [u8; 32] = sha256(&master_key);
        println!(" clé master : {:?},\n hash clé: {:?}", master_key, hash_master_key);

        /* Concaténation des de la clé et du hash */
        let mut master_key_2: Vec<u8> = vec![];
        master_key_2.extend_from_slice(&master_key);
        master_key_2.extend_from_slice(&hash_master_key);

        /*Chiffrement clé_master_2 */
        let kdf_key = AesKey::new_encrypt(&kdf_client).expect("wrap kdf n'a pas focntionner");
        let mut master_key_2_encrypted = [0u8; 72];
        let _ = wrap_key(&kdf_key, None, &mut master_key_2_encrypted, &master_key_2).expect("Problème lors du chiffrement de la clé master 2");

        /* enregistrer sur un format hexadécimal */
        return hex::encode(&master_key_2_encrypted);
    }

    pub async fn signin(&mut self, login:Login) -> Result<String, LoginState>{
         /* Initialisation des paramètre de connection à la base de donnée */
        let opt = mysql::MySqlConnectOptions::new().host("127.0.0.1").password("mypass").port(3306).username("root").database("strongholder");
        let mut connection = mysql::MySqlConnection::connect_with(&opt).await.unwrap();
        
        /* Récupération clé master 2 */
        let query = sqlx::query_as("SELECT id, encrypt_master_key_2 FROM Credentials WHERE username=?").bind(login.username.as_str());
        let result: Vec<MysqlCredentials> = query.fetch_all(&mut connection).await.expect("Une erreur c'est produite");

        /* Vérification si l'utilisateur existe */
        if result.len() != 1 {
            return Err(LoginState::NotSignup);
        }

        /* Création de la clé dériver */
        let kdf_client:[u8; 32]  = create_kdf(&login.password, &login.username);

        /* Convertion hex to bytes */
        let master_key_2_encrypted = hex::decode(result[0].encrypt_master_key_2.clone()).expect("Problème lors de la convertion hex to byte");

        /* Vérification du mot de passe */
        let kdf_key = AesKey::new_decrypt(&kdf_client).expect("wrap kdf n'a pas focntionner");
        let mut master_key_2 = [0u8; 64];
        let _ = match unwrap_key(&kdf_key, None, &mut master_key_2, &master_key_2_encrypted){
            Err(_)=> return Err(LoginState::InvalidPassword),
            Ok(o)=>o
        };

        /* Renvoyer le cookie JWT */
        let credentials = Credentials{id:result[0].id.clone(), kdf:hex::encode(kdf_client)};
        Ok(self.create_token(credentials))
    }

    fn create_token(&self, credentials: Credentials) -> String{
        let header = Header::new(Algorithm::HS384);
        let token = match encode(&header, &credentials, &EncodingKey::from_secret("secret".as_ref())){
            Ok(token) => token,
            Err(_)=> "erreur".to_string()
        };
        token
    }
}
