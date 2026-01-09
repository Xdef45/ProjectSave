use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::{post,web, App, HttpServer,Result, cookie::Cookie};
use crate::authentification::auth::{Auth, Login};
use actix_files::NamedFile;
#[post("/get_repot_key")]
async fn get_repot_key() -> Result<NamedFile>{
    let file = NamedFile::open_async("ssh_key").await
    .expect("imposible d'ouvrir le fichier")
    .set_content_disposition(header::ContentDisposition{
        disposition: header::DispositionType::Attachment,
        parameters: vec![header::DispositionParam::Filename("ping.txt".to_string())]
    });
    Ok(file)
}