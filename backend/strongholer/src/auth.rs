use sqlx::{mysql, Connection};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

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

pub struct Auth;
impl Auth {
    
    pub async fn get_access(login: Login) {
        let opt = mysql::MySqlConnectOptions::new().host("127.0.0.1").password("mypass").port(3306).username("root").database("strongholder");
        let mut connection = mysql::MySqlConnection::connect_with(&opt).await.unwrap();

        
        sqlx::query("INSERT INTO Credentials (id, KDF) VALUES(?,?)").bind(&"150").bind(&"test")
        .execute(&mut connection).await.unwrap();
    }
    pub async fn get_token() -> String{
        let header = Header::new(Algorithm::HS384);
        let credentials = Credentials{id:32, kdf:"test".to_string()};
        let token =match encode(&header, &credentials, &EncodingKey::from_secret("secret".as_ref())){
            Ok(token) => token,
            Err(_)=> "erreur".to_string()
        };
        token
    }


}