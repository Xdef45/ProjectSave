use actix_web::{HttpRequest, HttpResponse, cookie::Cookie, post,web, App, HttpServer};
mod authentification;
use crate::authentification::auth::{Auth, BearerState};
mod script;
use serde_json;

mod route;
use crate::route::{signup, signin, get_repot_key, send_ssh_key};

#[post("/imaconnected")]
async fn imaconnected(req: HttpRequest, auth: web::Data<Auth>) -> HttpResponse{
    if let Some(cookie) = req.cookie("Bearer"){
        let (bearer_state, (result, credentials)) = auth.validation(cookie.value().to_string());
        if bearer_state == BearerState::Valid {
            return HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&credentials).expect("Convertion de struct à string a échoué"))
        }
        if bearer_state == BearerState::Expired{
            let cookie = Cookie::build("Bearer", match result{
                Some(cookie)=>cookie,
                None=>"Rien".to_string()
            })
                .path("/")
                .secure(true)
                .http_only(true)
                .finish();
            return HttpResponse::Ok()
            .append_header(("Set-Cookie", cookie.to_string()))
            .content_type("application/json")
            .body(serde_json::to_string(&credentials).expect("Convertion de struct à string a échoué"))
        }
        if bearer_state == BearerState::Error{
            return HttpResponse::Ok()
            .body(match result{
                Some(res)=>res,
                None=>"Rien".to_string()
            })
        }
        return HttpResponse::BadRequest().body("Erreur inconnue")
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
