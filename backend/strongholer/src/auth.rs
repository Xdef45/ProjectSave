use sqlx::{mysql, Connection};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use crate::kdfpassword::base64_full_hash;

#[derive(Deserialize)]
pub struct Login{
    pub username: String,
    pub password: String
}
#[derive(Debug, Serialize, Deserialize)]
struct Credentials{
    id: u32,
    kdf: String
}
#[derive(Debug, Serialize, Deserialize)]
pub enum LoginState{
    AlreadyExist,

}

pub struct Auth;
impl Auth {
    
    pub async fn signup(&self, login: Login) -> Result<String, LoginState> {
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
        let kdf_client: String = base64_full_hash(&login.password, &login.username);
        let query = sqlx::query("INSERT INTO Credentials (username, encrypt_master_key_2) VALUES(?,?)")
        .bind(login.username.as_str())
        .bind(kdf_client);
        query.execute(&mut connection).await;
        
        /* Renvoyer le cookie JWT */
        let credentials = Credentials{id:32, kdf:"test".to_string()};
        Ok(self.create_token(credentials))
    }

    pub fn create_token(&self,credentials: Credentials) -> String{
        let header = Header::new(Algorithm::HS384);
        let token =match encode(&header, &credentials, &EncodingKey::from_secret("secret".as_ref())){
            Ok(token) => token,
            Err(_)=> "erreur".to_string()
        };
        token
    }


}
