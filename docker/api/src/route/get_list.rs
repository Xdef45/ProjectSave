use actix_web::{post, HttpResponse, HttpRequest, web, Result, FromRequest};
use crate::authentification::auth::Auth;
use crate::error::APIError;
use crate::borg_script::list_archive::{list_archive, Archives};
use serde::Deserialize;
use use serde_json;

#[derive(Deserialize)]
struct Archive{
    archive_name: String
}


#[post("/get_list")]
async fn get_list(req: HttpRequest, auth: web::Data<Auth>, body: String)->Result<HttpResponse, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let (_, (_, credentials)) = auth.validation(cookie.value().to_string());
    let Some(credentials) = credentials else {
        return Err(APIError::NoAuthAppData)
    };

    let archive: Archive = match serde_json::from_str(body){
        Ok(o)=>o,
        Err(_)=>{
            println!("Erreur lors de la conversion en json dans get_list");
            return APIError::Json
        }
    };

    match auth.restore_master_key_2_file(&credentials).await{
        Ok(_)=>{
            let archives = match list_archive(credentials.id, auth.ssh_connexion.clone()).await{
                Ok(archives)=> return Ok(HttpResponse::Ok().json(archives)),
                Err(e)=>{
                    println!("Erreur list archive");
                    return Err(e)}
            };
        },
        Err(e)=>{
            println!("Erreur restore master key");
            return Err(e)}
    };
    
}