use openssh::Session;
use std::sync::Arc;

pub async fn list_archive(uuid: String, ssh_connexion: Arc<Session>){
    let output = match ssh_connexion.command("sudo").args(["/usr/local/sbin/list.sh", uuid.as_str()]).output().await{
        Ok(o)=>o,
        Err(_)=>{println!("script list échoué");return}
    };
    println!("ici {}\n Erreur: {}",String::from_utf8(output.stdout).expect("UFT8"), String::from_utf8(output.stderr).expect("UTF8"));
}