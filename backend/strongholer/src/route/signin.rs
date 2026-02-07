use actix_web::{post,web, cookie::Cookie, HttpResponse};
use crate::authentification::auth::{Auth, Login, LoginState};

#[post("/signin")]
async fn signin(id: web::Json<Login>, auth: web::Data<Auth>) -> HttpResponse{
    let login= Login{
        username: id.username.clone(), 
        password: id.password.clone()
    };
    let token = match auth.signin(login).await{
        Ok(token)=>token,
        Err(e)=>{
            if e == LoginState::NotSignup{
                return HttpResponse::BadRequest().body("L'utilisateur n'est pas enregistré")
            }else{
                return HttpResponse::BadRequest().body("Erreur inconnue")
            }

        }
    };
    let cookie = Cookie::build("Bearer", token)
    .path("/")
    .secure(true)
    .http_only(true)
    .finish();
    HttpResponse::Ok()
    .append_header(("Set-Cookie", cookie.to_string()))
    .body("")
}