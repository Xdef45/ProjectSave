use actix_web::{HttpRequest, HttpResponse};
use actix_web::{post,web, App, HttpServer, Responder, Result, cookie::Cookie};
use serde::{Deserialize, Serialize};
mod kdfpassword;
use kdfpassword::base64_full_hash;
mod auth;
use auth::Auth;
use auth::Login;



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
async fn signup(id: web::Json<Login>) -> HttpResponse{
    let login= Login{
        username: id.username.clone(), 
        password: id.password.clone()
    };
    let token = Auth.signup(login).await.expect("Le token n'as pas pu se créer");
    let cookie = Cookie::build("jwt", token)
    .path("/")
    .secure(true)
    .http_only(true)
    .finish();
    HttpResponse::Ok()
    .append_header(("Set-Cookie", cookie.to_string()))
    .body("")
    
}


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
