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

    let (_, (_, credentials)) = match auth.validation(cookie.value().to_string()){
        Ok(res)=> res,
        Err(e)=>return Err(e)
    };
    let restore_file = match restore(credentials.id, archive.archive_name.clone(), auth.ssh_connexion.clone(), auth.sftp_connexion.clone()).await{
        Ok(f)=>f,
        Err(a)=>{
            return Err(a);
        }
    };
    let reader = TokioCompatFile::from(restore_file);
    let stream = StreamBuffer::new(reader);
    return Ok(HttpResponse::Ok().streaming(stream))
}