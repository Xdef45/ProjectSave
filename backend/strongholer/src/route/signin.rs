use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::{post,web, App, HttpServer,Result, cookie::Cookie};
use crate::authentification::auth::{Auth, Login};

#[post("/signin")]
async fn signin(id: web::Json<Login>) -> HttpResponse{
    let login= Login{
        username: id.username.clone(), 
        password: id.password.clone()
    };
    let token = Auth.signin(login).await.expect("Le token n'as pas pu se créer");
    let cookie = Cookie::build("Bearer", token)
    .path("/")
    .secure(true)
    .http_only(true)
    .finish();
    HttpResponse::Ok()
    .append_header(("Set-Cookie", cookie.to_string()))
    .body("")
}