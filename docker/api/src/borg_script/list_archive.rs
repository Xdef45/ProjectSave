use openssh::Session;
use std::sync::Arc;
use serde_json;
use serde::{Deserialize, Serialize};
use crate::error::APIError;

#[derive(Serialize, Deserialize)]
pub struct ArchiveData{
    pub archive: String,
    pub time: String
}
#[derive(Serialize, Deserialize)]
pub struct Archives{
    pub archives: Vec<ArchiveData>
}

pub async fn list_archive(uuid: &String, ssh_connexion: Arc<Session>,)->Result<Archives, APIError>{
    let output = match ssh_connexion.command("sudo").args([String::from("/usr/local/sbin/list.sh"), uuid.to_string()]).output().await{
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
        println!("Erreur lors du listing des archives\nstdout {}\n stderr: {}", &stdout, &stderr);
        return Err(APIError::Script)
    }

    let archives: Archives = serde_json::from_str(&stdout).expect("serde_json");
    return Ok(archives)
}

#[derive(Serialize, Deserialize)]
pub struct ArchiveContent{
    pub archive_name: String,
    pub archive_content: Vec<ArchiveFile>
}
#[derive(Serialize, Deserialize)]
pub struct ArchiveFile{
    r#type: String,
    pub path: String,
    mtime: String,
    size:u64
}


pub async fn list_archive_content(uuid: &String, ssh_connexion: Arc<Session>, archive_name:&String)->Result<ArchiveContent, APIError>{
    let output = match ssh_connexion.command("sudo").args([String::from("/usr/local/sbin/list.sh"), uuid.to_string(), archive_name.clone()]).output().await{
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
        println!("Erreur lors du listing du contenu de l'archive {}\nstdout {}\n stderr: {}", archive_name ,&stdout, &stderr);
        return Err(APIError::Script)
    }
    let mut archive_content = Vec::<ArchiveFile>::new();
    for line in stdout.split('\n') {
        if line.trim().is_empty() {
            continue;
        }
        let archive_file: ArchiveFile = match serde_json::from_str(line) {
            Ok(a)=>a,
            Err(_)=>{
                println!("Erreur lors de la conversion string to ArchiveFile list_archive");
                return Err(APIError::Json);
            }
        };
        archive_content.push(archive_file);
    }

    return Ok(ArchiveContent{archive_name: archive_name.to_string(), archive_content: archive_content})
}
