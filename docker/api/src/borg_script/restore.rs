use openssh::Session;
use std::sync::Arc;
use crate::error::APIError;
use openssh_sftp_client::file::TokioCompatFile;
use openssh_sftp_client::{file::File, Sftp};
const CLIENT_DIRECTORY: &str = "/srv/repos";

pub async fn restore(uuid: String, archive:String, ssh_connexion: Arc<Session>, sftp_connexion: Arc<Sftp>)->Result<File, APIError>{
    let output = match ssh_connexion.command("sudo").args(["/usr/local/sbin/restore.sh", uuid.as_str(), archive.as_str()]).output().await{
        Ok(o)=>o,
        Err(_)=>{println!("Erreur ssh command restore");return Err(APIError::Ssh)}
    };
    let stdout = match String::from_utf8(output.stdout.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stdout UTF8 restore");
            return Err(APIError::UTF8)
        }
    };
    let stderr = match String::from_utf8(output.stderr.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stderr UTF8 restore");
            return Err(APIError::UTF8)
        }
    };
    if ! output.status.success(){
        println!("Erreur lors du restore\nstdout {}\n stderr: {}", &stdout, &stderr);
        return Err(APIError::Script)
    }
    let file_restore_path = format!("{}/{}/restore/{}.tar.gz",CLIENT_DIRECTORY, uuid, archive);
    match sftp_connexion.open(file_restore_path).await{
        Ok(f)=>return Ok(f),
        Err(_)=>{
            println!("Erreur Sftp connexion restore");
            return Err(APIError::Sftp)
        }
    }
}