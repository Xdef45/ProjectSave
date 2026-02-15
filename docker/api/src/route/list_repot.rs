use actix_web::{post, HttpResponse, HttpRequest, web, Result,HttpResponseBuilder, http::StatusCode};
use crate::authentification::auth::Auth;
use crate::error::APIError;
use serde_json;
const CLIENT_DIRECTORY: &str = "/srv/repos"; 
use tokio::process::Command;
const MAX_FILE_SIZE_SSH_KEY: usize = 50 * 1024 * 1024;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
struct ListRepot{
    archives: Vec<String>,

}

#[post("/list_repot")]
async fn list_repot(req: HttpRequest, auth: web::Data<Auth>)->Result<HttpResponse, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let (_, (_, credentials)) = auth.validation(cookie.value().to_string());
    let Some(credentials) = credentials else {
        return Err(APIError::NoAuthAppData)
    };

    auth.restore_master_key_2_file(&credentials).await;

    let directory_user = format!("{}/{}/repo",CLIENT_DIRECTORY,credentials.id);
    let output = match Command::new(format!("sudo"))
    .args(["-u", &credentials.id, "borg","list", "--json", &directory_user])
    .output().await{
        Ok(o)=> o,
        Err(e)=> {
            println!("L'installation de la clé ssh client n'a pas fonctionné {}", e.to_string());
            return Err(APIError::Script)
        }
    };

    Auth::delete_master_key_2_file(&credentials).await;

    let result = match String::from_utf8(output.stdout){
        Ok(o)=>o,
        Err(e)=> {println!("{}", e.to_string()); return Err(APIError::ConversionVecToString)}
    };
    println!("Erreur : {}\n Sortie : {}", String::from_utf8(output.stderr).expect("msg"), &result);
    let result: ListRepot = serde_json::from_str(&result).expect("msg");
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(result))
}