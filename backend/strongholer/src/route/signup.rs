use actix_web::{post,web, cookie::Cookie,HttpResponse};
use crate::authentification::auth::{Auth, Login,LoginState};

/*S'incrire */
#[post("/signup")]
async fn signup(id: web::Json<Login>) -> HttpResponse{
    let login= Login{
        username: id.username.clone(), 
        password: id.password.clone()
    };
    println!("Reçus");
    let response = match Auth.signup(login).await {
        Ok(token ) => {
            let cookie = Cookie::build("Bearer", token)
                .path("/")
                .secure(true)
                .http_only(true)
                .finish();
            HttpResponse::Ok()
                .append_header(("Set-Cookie", cookie.to_string()))
                .finish()
        },
        Err(state) => {
            if state == LoginState::AlreadyExist {
                HttpResponse::BadRequest().body("L'utilisateur existe déjà")
            }else {
                HttpResponse::BadRequest().body("Erreur inconnue")
            }
        }
    };
    response
    
}