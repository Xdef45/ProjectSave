use actix_web::{post, HttpResponse, HttpRequest, web};
use crate::authentification::auth::Auth;
use crate::borg_script::install_client_key::install_client_key;
use crate::error::APIError;
use serde::Deserialize;

#[post("/restore")]
async fn restore(req: HttpRequest, auth: web::Data<Auth>)-> Result<HttpResponse, APIError>{

}