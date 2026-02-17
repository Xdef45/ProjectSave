use actix_web::{post, HttpResponse, HttpRequest, web};
use crate::authentification::auth::Auth;
use crate::borg_script::install_client_key::install_client_key;
use crate::error::APIError;
use serde::Deserialize;

#[derive(Deserialize)]
struct SshKey{
    ssh: String
}


#[post("/send_ssh_key")]
async fn send_ssh_key(req: HttpRequest, ssh_key: web::Json<SshKey>, auth: web::Data<Auth>)->Result<HttpResponse, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let credentials = Auth::decode_token(cookie.value())?;
    /* Upload du fichier */
    let filepath= format!("/srv/repos/api/{}.pub", credentials.id,);
    install_client_key(credentials.id, &ssh_key.ssh, filepath, auth.ssh_connexion.clone(), auth.sftp_connexion.clone()).await;
    return Ok(HttpResponse::Ok().finish())
}