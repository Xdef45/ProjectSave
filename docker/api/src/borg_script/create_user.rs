use openssh::Session;
use std::sync::Arc;
use openssh_sftp_client::Sftp;
use crate::error::APIError;
const CLIENT_DIRECTORY: &str = "/srv/repos";

pub async fn create_user(uuid:&String, ssh_connexion: Arc<Session>)-> Result<(), APIError>{
    // Execution du script create_user.sh
    let script_path=String::from("/usr/local/sbin/create_user.sh");
    let output = match ssh_connexion.command("sudo").args([&script_path, &uuid]).output().await{
        Ok(o)=>o,
        Err(e)=>{
            println!("Erreur lors du create user{}", e.to_string());
            return Err(APIError::Ssh)
        }
    };

    let stdout = match String::from_utf8(output.stdout.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stdout UTF8 create_user");
            return Err(APIError::UTF8)
        }
    };
    let stderr = match String::from_utf8(output.stderr.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stderr UTF8 create_user");
            return Err(APIError::UTF8)
        }
    };
    if ! output.status.success(){
        println!("Erreur lors de la création du user\nstdout {}\n stderr: {}", &stdout, &stderr);
        return Err(APIError::Script)
    }
    return Ok(())
    
}

pub async fn get_master_key_1_encrypted(uuid:&String, ssh_connexion: Arc<Session>, sftp_connexion: Arc<Sftp>)-> Result<Vec<u8>, APIError>{
        // Ouverture de la clé borg 1
    let path_key = format!("{}/{}/bootstrap/{}.gpg", CLIENT_DIRECTORY,uuid, uuid);
    println!("{}", path_key);
    let mut master_key_file = match sftp_connexion.open(&path_key).await {
        Ok(f)=>f,
        Err(_)=> {
            println!("Erreur lors de l'ouverture de la clé borg 1");
            return Err(APIError::Sftp)}
    };

    // Récupération de la taille de la clé
    let master_key_metadata = match master_key_file.metadata().await{
        Ok(meta)=>meta,
        Err(_)=>{
            println!("Erreur lors de la lecture des metadata clé borg 1 get_master_key_1_encrypted");
            return Err(APIError::Metadata)
        }
    };
    let master_key_len:usize = match master_key_metadata.len(){
        Some(size)=>match size.try_into(){
            Ok(size)=>size,
            Err(_)=>{
                println!("Erreur lors de la converion u64 to usize get_master_key_1_encrypted");
                return Err(APIError::Usize)
            }
        },
        None=>{
            println!("Longueur de laclé Borg 1 vide");
            return Err(APIError::NoFile)
        }
    };

    // Récupération des données de la clé borg
    let buf= bytes::BytesMut::with_capacity(master_key_len);
    let master_key_byte = master_key_file.read_all(master_key_len, buf).await.expect("read all échoué");
    // Supression de la clé borg
    /* 
    let output = match ssh_connexion.command("rm").arg(path_key).output().await{
        Ok(o)=>o,
        Err(_)=> {
            println!("Erreur command ssh rm key");
            return Err(APIError::Ssh)
        }
    };
    let stdout = match String::from_utf8(output.stdout.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stdout UTF8 get_master_key_1_encrypted");
            return Err(APIError::UTF8)
        }
    };
    let stderr = match String::from_utf8(output.stderr.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stderr UTF8 get_master_key_1_encrypted");
            return Err(APIError::UTF8)
        }
    };
    if ! output.status.success(){
        println!("Erreur lors de la supressionde la clé borg\nstdout {}\n stderr: {}", &stdout, &stderr);
        return Err(APIError::Script)
    }
    */
    return Ok(master_key_byte.to_vec())
}

pub async fn get_master_key_2(uuid:&String, ssh_connexion: Arc<Session>, sftp_connexion: Arc<Sftp>)->Result<String, APIError>{
    // Ouverture de la clé borg
    let path_key = format!("{}/{}/.config/borg/keys/srv_repos_{}_repo", CLIENT_DIRECTORY,uuid, uuid).to_string();
    println!("{}", path_key);
    let mut master_key_file = match sftp_connexion.open(&path_key).await {
        Ok(f)=>f,
        Err(_)=> {
            println!("Erreur lors de l'ouverture de la clé borg");
            return Err(APIError::Sftp)}
    };

    // Récupération de la taille de la clé
    let master_key_metadata = match master_key_file.metadata().await{
        Ok(meta)=>meta,
        Err(_)=>{
            println!("Erreur lors de la lecture des metadata clé borg create_user");
            return Err(APIError::Metadata)
        }
    };
    let master_key_len:usize = match master_key_metadata.len(){
        Some(size)=>match size.try_into(){
            Ok(size)=>size,
            Err(_)=>{
                println!("Erreur lors de la converion u64 to usize create_user");
                return Err(APIError::Usize)
            }
        },
        None=>{
            println!("Longueur de laclé Borg vide");
            return Err(APIError::NoFile)
        }
    };

    // Récupération des données de la clé borg
    let buf= bytes::BytesMut::with_capacity(master_key_len);
    let master_key_byte = master_key_file.read_all(master_key_len, buf).await.expect("read all échoué");
    let Ok(master_key) = String::from_utf8(master_key_byte.to_vec())else {
        println!("Erreur lors de la convertion byte to string create_user");
        return Err(APIError::UTF8)
    };
    // Supression de la clé borg
    let output = match ssh_connexion.command("rm").arg(path_key).output().await{
        Ok(o)=>o,
        Err(_)=> {
            println!("Erreur command ssh rm key");
            return Err(APIError::Ssh)
        }
    };
    let stdout = match String::from_utf8(output.stdout.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stdout UTF8 get_master_key_2");
            return Err(APIError::UTF8)
        }
    };
    let stderr = match String::from_utf8(output.stderr.clone()){
        Ok(out)=>out,
        Err(_)=>{
            println!("Erreur conversion stderr UTF8 get_master_key_2");
            return Err(APIError::UTF8)
        }
    };
    if ! output.status.success(){
        println!("Erreur lors de la supressionde la clé borg\nstdout {}\n stderr: {}", &stdout, &stderr);
        return Err(APIError::Script)
    }
    return Ok(master_key)
}