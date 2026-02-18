use openssh::Session;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::APIError;
use crate::borg_script::list_archive::{ArchiveContent, list_archive, list_archive_content};

#[derive(Serialize, Deserialize)]
pub struct Logs{
    logs: Vec<String>
}

pub async fn list_log_content(uuid: &String, ssh_connexion: Arc<Session>)->Result<Logs, APIError>{
    let mut archives_content = Vec::<ArchiveContent>::new();
    let archives = list_archive(uuid, ssh_connexion.clone()).await?;
    for archive_name in &archives.archives{
        archives_content.push(list_archive_content(uuid, ssh_connexion.clone(), &archive_name.archive).await?)
    }
    let mut logs_path = Vec::<String>::new();
    for archive in archives_content{
        logs_path.push(get_log_filename(archive)?);
    } 
    let mut logs = Logs{logs: Vec::<String>::new()};
    
    for i in 0..logs_path.len(){
        let _ = retore_log_file(uuid, ssh_connexion.clone(), &archives.archives[i].archive, &logs_path[i]).await?;
        let content = get_log_file(uuid, ssh_connexion.clone(), &archives.archives[i].archive, &logs_path[i]).await?;
        let _ = delete_log_file(uuid, ssh_connexion.clone(), &archives.archives[i].archive, &logs_path[i]).await?;
        logs.logs.push(content);
    }
    
    return Ok(logs)
}


fn get_log_filename(archive: ArchiveContent)->Result<String, APIError>{
    let log_filename = format!("{}_logs",archive.archive_name);
    let mut log_path = None;
    for file in archive.archive_content{
        if file.path.contains(&log_filename){
            log_path = Some(file.path);
            break;
        }
    }
    match log_path {
        Some(path)=>Ok(path),
        None=>{
            println!("Lors du listing des logs, le fichier {} n'a pas été trouvé dans l'archive {}", log_filename, archive.archive_name);
            Err(APIError::Script)}
    }
}

async fn retore_log_file(uuid: &String, ssh_connexion:Arc<Session>, archive_name: &String, log_path: &String)->Result<(), APIError>{
    // restoration du fichier
    let script_name = String::from("/usr/local/sbin/restore.sh");
    let output = match ssh_connexion.command("sudo").args([&script_name, uuid, archive_name, &log_path]).output().await{
        Ok(o)=>o,
        Err(_)=>{println!("connexion ssh erreur");return Err(APIError::Ssh)}
    };
    let stdout = match String::from_utf8(output.stdout.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stdout UTF8 retore_log_file");
            return Err(APIError::UTF8)
        }
    };
    let stderr = match String::from_utf8(output.stderr.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stderr UTF8 retore_log_file");
            return Err(APIError::UTF8)
        }
    };
    if ! output.status.success(){
        println!("Erreur lors de la restoration du fichier de logs {}\nstdout {}\n stderr: {}", log_path ,&stdout, &stderr);
        return Err(APIError::Script)
    }
    Ok(())
}

async fn get_log_file(uuid: &String, ssh_connexion:Arc<Session>, archive_name: &String, log_path: &String)->Result<String, APIError>{
    // restoration du fichier
    let output = match ssh_connexion.command("cat").arg(format!("/srv/repos/{}/restore/{}_{}.log", uuid, archive_name, uuid)).output().await{
        Ok(o)=>o,
        Err(_)=>{println!("connexion ssh erreur");return Err(APIError::Ssh)}
    };
    let stdout = match String::from_utf8(output.stdout.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stdout UTF8 get_log_file");
            return Err(APIError::UTF8)
        }
    };
    let stderr = match String::from_utf8(output.stderr.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stderr UTF8 get_log_file");
            return Err(APIError::UTF8)
        }
    };
    if ! output.status.success(){
        println!("Erreur lors de la récupération du fichier {}\nstdout {}\n stderr: {}", log_path ,&stdout, &stderr);
        return Err(APIError::Script)
    }
    Ok(stdout)
}
async fn delete_log_file(uuid: &String, ssh_connexion:Arc<Session>, archive_name: &String, log_path: &String)->Result<(), APIError>{
    // restoration du fichier
    let output = match ssh_connexion.command("rm").arg(format!("/srv/repos/{}/restore/{}_{}.log", uuid, archive_name, uuid)).output().await{
        Ok(o)=>o,
        Err(_)=>{println!("connexion ssh erreur");return Err(APIError::Ssh)}
    };
    let stdout = match String::from_utf8(output.stdout.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stdout UTF8 delete_log_file");
            return Err(APIError::UTF8)
        }
    };
    let stderr = match String::from_utf8(output.stderr.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stderr UTF8 delete_log_file");
            return Err(APIError::UTF8)
        }
    };
    if ! output.status.success(){
        println!("Erreur lors de la supression du fichier {}\nstdout {}\n stderr: {}", log_path ,&stdout, &stderr);
        return Err(APIError::Script)
    }
    Ok(())
}