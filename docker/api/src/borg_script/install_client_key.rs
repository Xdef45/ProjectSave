use openssh::Session;
use std::sync::Arc;
use openssh_sftp_client::{Sftp, file};
use bytes::BytesMut;
use crate::authentification::auth::LogupState;
const CLIENT_DIRECTORY: &str = "/srv/repos";

pub async fn install_client_key(uuid:String, filepath:String, ssh_connexion: Arc<Session>){
    let script_path=String::from("/usr/local/sbin/install_client_key.sh");
    match ssh_connexion.command("sudo").args([&script_path, &uuid, &filepath]).output().await{
        Ok(o)=>println!("script create_user"),
        Err(e)=>println!("{}", e.to_string())
    };
}