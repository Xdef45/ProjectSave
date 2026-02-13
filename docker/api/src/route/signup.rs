use actix_web::{post,web, cookie::Cookie,HttpResponse};
use crate::authentification::auth::{Auth, Login,LogupState};

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
            match state{
                LogupState::AlreadyExist => {
                    println!("User: {} already exist", id.username);
                    return HttpResponse::BadRequest().body("1")},
                LogupState::UsernameTooShort => {
                    println!("User: {} username too short", id.username);
                    return HttpResponse::BadRequest().body("2")},
                LogupState::InvalidPassword => {
                    println!("User: {} invalid password", id.username);
                    return HttpResponse::BadRequest().body("3")},
                LogupState::PasswordTooShort => {
                    println!("User: {} password too short", id.username);
                    return HttpResponse::BadRequest().body("4")},
                LogupState::SpecialCharMissing => {
                    println!("User: {} password special char missing", id.username);
                    return HttpResponse::BadRequest().body("5")},
                LogupState::MajusculeMissing => {
                    println!("User: {} password majuscule missing", id.username);
                    return HttpResponse::BadRequest().body("6")},
                LogupState::NumberMissing => {
                    println!("User: {} password number missing", id.username);
                    return HttpResponse::BadRequest().body("7")},
                LogupState::KDFError => {
                    println!("User: {} error during kdf creation", id.username);
                    return HttpResponse::BadRequest().body("8")},
                LogupState::ScriptError => {
                    println!("User: {} error during script", id.username);
                    return HttpResponse::BadRequest().body("9")}
            }
        }
    };

    let cookie = Cookie::build("Bearer", token)
    .path("/")
    .secure(true)
    .http_only(true)
    .finish();
    println!("User: {} signup", id.username);
    HttpResponse::Ok()
        .append_header(("Set-Cookie", cookie.to_string()))
        .finish()
    
}