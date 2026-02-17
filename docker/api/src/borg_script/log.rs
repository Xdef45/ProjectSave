use openssh::Session;
use std::sync::Arc;
use serde_json;
use serde::{Deserialize, Serialize};
use crate::error::APIError;
use crate::borg_script::list_archive::{ArchiveContent, list_archive, list_archive_content, Archives};

#[derive(Serialize, Deserialize)]
pub struct Logs{
    Logs: Vec<String>
}

pub async fn list_log_content(uuid: &String, ssh_connexion: Arc<Session>)->Result<Logs, APIError>{
    let script_name = String::from("/usr/local/sbin/list.sh");
    let archives_content = Vec::<ArchiveContent>::new();
    let archives = list_archive(uuid, ssh_connexion).await?;
    for archive_name in archives.archives{
        archives_content.push(list_archive_content(uuid, ssh_connexion, archive_name.archive).await?)
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
let output = match ssh_connexion.command("sudo").args([&script_name, uuid, &archive_name.archive, ]).output().await{
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
    return Ok(ArchiveContent{archive_content: archive_content})
}


fn get_log_filename(archive: ArchiveContent){
    let log_filename = archive.archive_name;
    let mut log_path = None;
    for file in archive.archive_content{
        if file.path.contains(&log_filename){
            log_path = Some(file.path);
        }
    }
}