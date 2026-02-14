use openssh::Session;
use std::sync::Arc;
use openssh_sftp_client::Sftp;
use bytes::BytesMut;
use crate::authentification::auth::LogupState;
const CLIENT_DIRECTORY: &str = "/srv/repos";

pub async fn create_user(uuid:&String, ssh_connexion: Arc<Session>, sftp_connexion: Arc<Sftp>)-> Result<String, LogupState>{
    let script_path=String::from("/usr/local/sbin/create_user.sh");
    let _ = match ssh_connexion.command("sudo").args([&script_path, &uuid]).output().await{
        Ok(o)=>{println!("script create_user");o},
        Err(e)=>{
            println!("{}", e.to_string());
            return Err(LogupState::ScriptError)
        }
    };
    let path_key = format!("{}/{}/.config/borg/keys/srv_repos_{}_repo", CLIENT_DIRECTORY,uuid, uuid).to_string();
    println!("{}", path_key);
    let mut master_key_file = match sftp_connexion.open(&path_key).await {
        Ok(f)=>{println!("srv_respos_ouvert");f},
        Err(_)=> return Err(LogupState::ScriptError)
    };
    let master_key_metadata = master_key_file.metadata().await.expect("metadata échoué");
    let master_key_len:usize = match master_key_metadata.len(){
        Some(size)=>size.try_into().expect("conversion usize"),
        None=> return Err(LogupState::ScriptError)
    };
    let mut buf= bytes::BytesMut::with_capacity(master_key_len);
    let master_key_byte = master_key_file.read_all(master_key_len, buf).await.expect("read all échoué");
    let master_key = String::from_utf8(master_key_byte.to_vec()).expect("conversion to string échoué");
    let _ = match ssh_connexion.command("rm").arg(path_key).output().await{
        Ok(o)=>{println!("script create_user");o},
        Err(e)=>{
            println!("{}", e.to_string());
            return Err(LogupState::ScriptError)
        }
    };
    return Ok(master_key)
}