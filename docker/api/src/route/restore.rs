use actix_web::{post, HttpResponse, HttpRequest, web};
use crate::authentification::auth::Auth;
use crate::borg_script::restore::restore;
use crate::error::APIError;
use crate::stream_http::stream_http::StreamBuffer;
use openssh_sftp_client::file::TokioCompatFile;
use serde::Deserialize;

#[derive(Deserialize)]
struct Restore{
    archive_name:String
}

#[post("/get_restore")]
async fn get_restore(req: HttpRequest, auth: web::Data<Auth>, archive: web::Json<Restore>)-> Result<HttpResponse, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let credentials=Auth::decode_token(cookie.value())?;
    let _ = auth.restore_master_key_2_file(&credentials).await?;
    let restore_file = restore(credentials.id.clone(), archive.archive_name.clone(), auth.ssh_connexion.clone(), auth.sftp_connexion.clone()).await?;
    let reader = TokioCompatFile::from(restore_file);
    let stream = StreamBuffer::new(reader);
    auth.delete_master_key_file(&credentials.id).await?;
    return Ok(HttpResponse::Ok().append_header(("Content-Disposition", format!("attachment; filename=\"{}.tar_gz\"", archive.archive_name))).streaming(stream))
}