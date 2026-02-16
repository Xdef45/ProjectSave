use actix_web::{post, HttpResponse, HttpRequest, web, Result, FromRequest};
use crate::authentification::auth::Auth;
use crate::error::APIError;
use crate::borg_script::list_archive::{list_archive, Archives};
use serde::Deserialize;
use serde_json;

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
    let archive: Archive = match serde_json::from_str(body.as_str()){
        Ok(o)=>o,
        Err(_)=>{
            println!("Erreur lors de la conversion en json dans get_list");
            return Err(APIError::Json)
        }
    };

    match auth.restore_master_key_2_file(&credentials).await{
        Ok(_)=>{
            let archive_script = if archive.archive_name.len() == 0{
                list_archive(credentials.id, auth.ssh_connexion.clone(), None).await
            }else{
                list_archive(credentials.id, auth.ssh_connexion.clone(), Some(archive.archive_name)).await
            };
            let archives = match archive_script{
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