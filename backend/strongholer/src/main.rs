use actix_web::middleware;
use actix_web::{HttpRequest, HttpResponse, post,web, App, HttpServer};
mod authentification;
use crate::authentification::auth::Auth;
use crate::authentification::middleware_auth;
mod script;
use serde_json;
mod error;

mod route;
use crate::route::{signup, signin, get_repot_key, send_ssh_key, list_repot};

#[post("/imaconnected")]
async fn imaconnected(req: HttpRequest, auth: web::Data<Auth>) -> HttpResponse{
    if let Some(cookie) = req.cookie("Bearer"){
        let (_, (_, credentials)) = auth.validation(cookie.value().to_string());
        return HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&credentials).expect("Convertion de struct à string a échoué"))
    }else{
        return HttpResponse::BadRequest().body("Vous n'avez pas de cookie de connection")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let auth = Auth::new().await;
    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(auth.clone()))
        .service(
            web::scope("/api")
            .wrap(middleware::from_fn(middleware_auth::authentification_middleware))
            .service(signup::signup)
            .service(signin::signin)
            .service(imaconnected)
            .service(send_ssh_key::send_ssh_key)
            .service(get_repot_key::get_repot_key)
            .service(list_repot::list_repot)
        )
    })
    .bind(("0.0.0.0", 8080)).expect("exit notime to play")
    .run()
    .await
}
