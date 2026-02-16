use openssh::Session;
use std::sync::Arc;
use serde_json;
use serde::{Deserialize, Serialize};
use crate::error::APIError;

#[derive(Serialize, Deserialize)]
pub struct ArchiveData{
    archive: String,
    time: String
}
#[derive(Serialize, Deserialize)]
pub struct Archives{
    archives: Vec<ArchiveData>
}

pub async fn list_archive(uuid: String, ssh_connexion: Arc<Session>, archive_name:Option<String>)->Result<Archives, APIError>{
    let output = match archive_name {
        Some(archive)=>ssh_connexion.command("sudo").args([String::from("/usr/local/sbin/list.sh"), uuid, archive]).output().await,
        None=>ssh_connexion.command("sudo").args([String::from("/usr/local/sbin/list.sh"), uuid]).output().await
    };
    let output = match output{
        Ok(o)=>o,
        Err(_)=>{println!("connexion ssh erreur");return Err(APIError::Ssh)}
    };
    let stdout = match String::from_utf8(output.stdout.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stdout UTF8 decrypt_master_2_key_create");
            return Err(APIError::UTF8)
        }
    };
    let stderr = match String::from_utf8(output.stderr.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stderr UTF8 decrypt_master_2_key_create");
            return Err(APIError::UTF8)
        }
    };
    if ! output.status.success(){
        println!("Erreur lors du listing des archive\nstdout {}\n stderr: {}", &stdout, &stderr);
        return Err(APIError::Script)
    }

    let archives: Archives = serde_json::from_str(&stdout).expect("serde_json");
    return Ok(archives)
}