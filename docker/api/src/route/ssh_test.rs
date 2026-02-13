use actix_web::{post,web, cookie::Cookie, HttpResponse};
use crate::authentification::auth::{Auth, Login, LoginState};
use openssh::{Session, KnownHosts};
use std::sync::Arc;

#[post("/ssh_test")]
async fn ssh_test(session: web::Data<Arc<Session>> ) -> HttpResponse{
    println!("yes");
    let ls = match session.command("ls").output().await{
        Ok(o)=>o,
        Err(_)=>return HttpResponse::Ok().finish()
    };
    println!("{}",String::from_utf8(ls.stdout).expect("msg"));
    return HttpResponse::Ok().finish();
}