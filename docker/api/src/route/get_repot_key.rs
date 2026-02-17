use actix_web::HttpResponse;
use actix_web::{HttpRequest, Result, post, web};
use openssh_sftp_client::file::TokioCompatFile;
use crate::error::APIError;
use crate::authentification::auth::Auth;
const CLIENT_DIRECTORY: &str = "/srv/repos";
use crate::stream_http::stream_http::StreamBuffer;


#[post("/get_repot_key")]
async fn get_repot_key(req: HttpRequest, auth: web::Data<Auth>) -> Result<HttpResponse, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let credentials = Auth::decode_token(cookie.value())?;
    let filepath = format!("{}/{}/bootstrap/{}.gpg", CLIENT_DIRECTORY, credentials.id,credentials.id);
    let repot_key = match auth.sftp_connexion.open(filepath).await{
        Ok(f)=>f,
        Err(_)=>return Err(APIError::Script)
    };
    let reader = TokioCompatFile::from(repot_key);
    let stream = StreamBuffer::new(reader);
    return Ok(HttpResponse::Ok().streaming(stream))
    
}
