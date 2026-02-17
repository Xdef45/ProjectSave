use actix_web::{post, HttpResponse, HttpRequest, web, Result, FromRequest};
use crate::authentification::auth::Auth;
use crate::error::APIError;
use crate::borg_script::list_archive::{Archives, ArchiveFile, list_archive, list_archive_content};
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

    let (_, (_, credentials)) = match auth.validation(cookie.value().to_string()){
        Ok(res)=> res,
        Err(e)=>return Err(e)
    };

    let _ = match auth.restore_master_key_2_file(&credentials).await{
        Ok(_)=>(),
        Err(e)=>return Err(e)
    };
    if body.len() == 0{
        match list_archive(&credentials.id, auth.ssh_connexion.clone()).await{
            Ok(archives)=> return Ok(HttpResponse::Ok().json(archives)),
            Err(e)=>{
                println!("Erreur list archive");
                return Err(e)}
        };
    }else{
        let archive: Archive = match serde_json::from_str(body.as_str()){
            Ok(o)=>o,
            Err(_)=>{
                println!("Erreur lors de la conversion en json dans get_list");
                return Err(APIError::Json)
            }
        };
        match list_archive_content(&credentials.id, auth.ssh_connexion.clone(), archive.archive_name).await{
            Ok(archive_files)=> return Ok(HttpResponse::Ok().json(archive_files)),
            Err(e)=>return Err(e)
        };
    };
            
    
}