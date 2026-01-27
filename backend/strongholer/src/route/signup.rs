use actix_web::{post,web, cookie::Cookie,HttpResponse};
use crate::authentification::auth::{Auth, Login,LoginState};

/*S'incrire */
#[post("/signup")]
async fn signup(id: web::Json<Login>, auth: web::Data<Auth>) -> HttpResponse{
    let login= Login{
        username: id.username.clone(), 
        password: id.password.clone()
    };

    // Création du Token
    let token = match auth.signup(login).await {
        Ok(token)=>token,
        Err(state)=>{
            if state == LoginState::AlreadyExist {
                return HttpResponse::BadRequest().body("L'utilisateur existe déjà")
            }else {
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
        .finish()
    
}