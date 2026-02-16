use actix_web::{post, HttpResponse, HttpRequest, web, Result};
use crate::authentification::auth::Auth;
use crate::error::APIError;
use crate::borg_script::list_archive::list_archive;


#[post("/get_list")]
async fn get_list(req: HttpRequest, auth: web::Data<Auth>)->Result<HttpResponse, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let (_, (_, credentials)) = auth.validation(cookie.value().to_string());
    let Some(credentials) = credentials else {
        return Err(APIError::NoAuthAppData)
    };
    match auth.restore_master_key_2_file(&credentials).await{
        Ok(_)=>{
            list_archive(credentials.id, auth.ssh_connexion.clone()).await;
            return Ok(HttpResponse::Ok().finish())
        },
        Err(e)=>return Err(e)
    };
    
}