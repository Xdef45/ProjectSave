use actix_web::{post, http::header, Result};
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