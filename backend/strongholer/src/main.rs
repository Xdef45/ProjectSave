use actix_web::{HttpRequest, HttpResponse};
use actix_web::{post,web, App, HttpServer};
mod authentification;
use crate::authentification::auth::{Auth};
mod script;

mod route;
use crate::route::{signup, signin, get_repot_key, send_ssh_key};

#[post("/imaconnected")]
async fn imaconnected(req: HttpRequest) -> HttpResponse{
    if let Some(cookie) = req.cookie("Bearer"){
        let _ = Auth.validation(cookie.value().to_string()).await.expect("Lors de la validation d'un cookie, une erreur est survenue");
    }
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        App::new().service(
            web::scope("/api")
            .service(signup::signup)
            .service(signin::signin)
            .service(imaconnected)
            .service(send_ssh_key::send_ssh_key)
            .service(get_repot_key::get_repot_key)
        )
    })
    .bind(("0.0.0.0", 8080)).expect("exit notime to play")
    .run()
    .await
}
