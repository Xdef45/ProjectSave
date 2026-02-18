use actix_web::{HttpRequest, Result, post,http::header::{ContentDisposition, DispositionType, DispositionParam}, web,HttpResponse};
use crate::error::APIError;
use crate::authentification::auth::Auth;
use crate::stream_http::stream_http::StreamBuffer2;


#[post("/get_repot_key")]
async fn get_repot_key(req: HttpRequest, auth: web::Data<Auth>) -> Result<HttpResponse, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let credentials = Auth::decode_token(cookie.value())?;
    let repot_key = auth.decrypt_master_1_key(&credentials).await?;
    let stream = StreamBuffer2::new(repot_key.clone());
    let content_disposition = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![
            DispositionParam::Filename(String::from(format!("{}.gpg", credentials.id)))
        ],
    };
    return Ok(HttpResponse::Ok().insert_header(content_disposition).streaming(stream))
}

