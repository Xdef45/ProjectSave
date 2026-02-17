use actix_web::{post, HttpResponse, web, Result};
use crate::authentification::auth::Auth;
use crate::error::APIError;
use crate::borg_script::ssh_pub_key_server::ssh_pub_key_server;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PubSshKey{
    ssh_pub: String
}


#[post("/get_ssh_pub_key_server")]
async fn get_ssh_pub_key_server(auth: web::Data<Auth>)-> Result<HttpResponse, APIError>{
    let ssh_key = ssh_pub_key_server(auth.ssh_connexion.clone()).await?;
    let ssh_pub_key = PubSshKey{ssh_pub: ssh_key};
    Ok(HttpResponse::Ok().json(ssh_pub_key))
}