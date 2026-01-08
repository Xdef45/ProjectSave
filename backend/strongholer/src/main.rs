use actix_web::{HttpRequest, HttpResponse};
use actix_web::{post,web, App, HttpServer, cookie::Cookie};
mod kdfpassword;
mod auth;
use auth::Auth;
use auth::Login;

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
    .finish()
    
}

#[post("/imaconnected")]
async fn imaconnected(req: HttpRequest) -> HttpResponse{
    if let Some(cookie) = req.cookie("jwt"){
        let _ = Auth.validation(cookie.value().to_string()).expect("Lors de la validation d'un cookie, une erreur est survenue");
    }
    HttpResponse::Ok().finish()
}

#[post("/signin")]
async fn signin(id: web::Json<Login>) -> HttpResponse{
    let login= Login{
        username: id.username.clone(), 
        password: id.password.clone()
    };
    let token = Auth.signin(login).await.expect("Le token n'as pas pu se créer");
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
            .service(signin)
            .service(imaconnected)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
