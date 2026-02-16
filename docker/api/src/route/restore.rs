use actix_web::{post, HttpResponse, HttpRequest, web};
use crate::authentification::auth::Auth;
use crate::borg_script::restore::restore;
use crate::error::APIError;
use serde::Deserialize;

#[post("/restore")]
async fn restore(req: HttpRequest, auth: web::Data<Auth>)-> Result<HttpResponse, APIError>{

}