use actix_web::HttpResponse;
use actix_web::{HttpRequest, Result, post, web};
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
    let stream = StreamBuffer2::new(repot_key);
    return Ok(HttpResponse::Ok().streaming(stream))
    
}
