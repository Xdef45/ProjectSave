use actix_web::{post,web, App, HttpServer, Responder, Result};
use serde::{Deserialize, Serialize};
mod kdfpassword;
use kdfpassword::base64_full_hash;
mod auth;
use auth::Auth;
use auth::login;

#[derive(Deserialize)]

#[derive(Serialize)]
struct MyObj{
    hash: String
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    id: u32,
    kdf: String
}


/*S'incrire */
#[post("/signup")]
async fn signup(id: web::Json<Login>) -> Result<impl Responder>{
    let password= &id.password;
    let salt = &id.username;
    let user: User = User{
        id:1,
        kdf: String::from(base64_full_hash(&password, &salt))
    };
    let login = auth::login {
        username: id.username.clone(),
        password: id.password.clone()
    };
    let _ = Auth::get_access().await;
    Ok(web::Json(user))
}
/*Se connecter */
#[post("/signin")]



#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        App::new().service(
            web::scope("/api")
            .service(signup)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
