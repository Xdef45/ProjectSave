use actix_web::middleware;
use actix_web::{HttpRequest, HttpResponse, post,web, App, HttpServer};
mod authentification;
use crate::authentification::auth::Auth;
use crate::authentification::middleware_auth;
use serde_json;
mod error;
mod route;
mod borg_script;
use crate::route::{get_list, get_repot_key, get_ssh_pub_key_server, send_ssh_key, send_ssh_key_tunnel, signin, signup};

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
    println!("connection db et ssh réussi");

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
            .service(send_ssh_key_tunnel::send_ssh_key_tunnel)
            .service(get_repot_key::get_repot_key)
            .service(get_list::get_list)
            .service(get_ssh_pub_key_server::get_ssh_pub_key_server)
        )
    })
    .bind(("0.0.0.0", 8080)).expect("exit notime to play")
    .run()
    .await
}
