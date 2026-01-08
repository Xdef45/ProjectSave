use actix_web::{HttpRequest, HttpResponse};
use actix_web::{post,web, App, HttpServer, cookie::Cookie};
mod authentification;
use crate::authentification::auth::{Login, Auth};
use crate::authentification::kdfpassword;
mod script;
use crate::script::borg_script;
use async_std::fs;
use async_std::prelude::*;
use actix_multipart::Multipart;

const MAX_FILE_SIZE_SSH_KEY: usize = 50 * 1024 * 1024;

/*S'incrire */
#[post("/signup")]
async fn signup(id: web::Json<Login>) -> HttpResponse{
    let login= Login{
        username: id.username.clone(), 
        password: id.password.clone()
    };
    let token = Auth.signup(login).await.expect("Le token n'as pas pu se créer");
    let cookie = Cookie::build("Bearer", token)
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
    if let Some(cookie) = req.cookie("Bearer"){
        let _ = Auth.validation(cookie.value().to_string()).expect("Lors de la validation d'un cookie, une erreur est survenue");
    }
    HttpResponse::Ok().finish()
}

#[post("sendsshkey")]
async fn send_ssh_key(mut payload: Multipart)->HttpResponse{
     while let Some(field) = payload.next().await {
        let mut field = field.expect("field invalide");

        // On ne garde JAMAIS le filename brut (risque path traversal)
        let cd = field.content_disposition();
        let filepath= "ssh_key".to_string();
        // Crée le fichier
        let mut f = fs::File::create(&filepath)
            .await.expect("Création du fichier à échouer");

        // Stream vers disque en chunks, sans charger en RAM
        let mut written: usize = 0;
        while let Some(chunk) = field.next().await {
            let chunk = chunk.expect("chunk incorrect");
            written = written.saturating_add(chunk.len());
            if written > MAX_FILE_SIZE_SSH_KEY {
                // Nettoyage si dépassement
                let _ = fs::remove_file(&filepath).await;
                   return HttpResponse::Ok().body("fichier tro gros");
            }
            f.write_all(&chunk)
                .await.expect("impossible d'écrire dans le fichier");
        }
    }
    HttpResponse::Ok().finish()
}
/* 
#[post("/getkey")]
async fn get_key() -> HttpResponse{
    let mut file = File::open("test.txt").await;
    let mut content = Vec::new();
    file.read_to_end(&mut content).await; /* .expect("Fichier ilisible");*/
    HttpResponse::Ok().finish()
}
*/
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


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        App::new().service(
            web::scope("/api")
            .service(signup)
            .service(signin)
            .service(imaconnected)
            .service(send_ssh_key)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
