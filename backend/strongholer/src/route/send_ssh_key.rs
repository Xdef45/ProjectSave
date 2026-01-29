use actix_web::{post, HttpResponse, HttpRequest, web};
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use crate::authentification::auth::Auth;
const CLIENT_DIRECTORY: &str = "/srv/repos"; 
use tokio::{process::Command, fs, io::AsyncWriteExt};
const MAX_FILE_SIZE_SSH_KEY: usize = 50 * 1024 * 1024;

#[post("send_ssh_key")]
async fn send_ssh_key(req: HttpRequest, mut payload: Multipart, auth: web::Data<Auth>)->HttpResponse{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return HttpResponse::Ok().body("Pas de cookie Bearer")
    };

    let (_, (_, credentials)) = auth.validation(cookie.value().to_string());
    let Some(credentials) = credentials else {
        return HttpResponse::Ok().body("Pas de credentials")
    };
    
    /* Upload du fichier */
    let filepath= format!("{}/{}/bootstrap/ssh-key.pub", CLIENT_DIRECTORY, credentials.id,);
    println!("{}",filepath);
    while let Some(field) = payload.next().await {
        let mut field = field.expect("field invalide");
        
        // Crée le fichier
        let mut f = fs::File::create(&filepath)
            .await.expect("Création du fichier à échouer");

        // Stream vers disque en chunks, sans charger en RAM
        let mut written: usize = 0;
        while let Some(chunk) = field.next().await {
            let chunk = chunk.expect("chunk incorrect");
            written = written.saturating_add(chunk.len());
            if written > MAX_FILE_SIZE_SSH_KEY {
                // Nettoyage si dépassement
                let _ = fs::remove_file(&filepath).await;
                   return HttpResponse::Ok().body("fichier tros gros");
            }
            f.write_all(&chunk)
                .await.expect("impossible d'écrire dans le fichier");
        }
    }

    /* Execution du script d'ajout de la clé ssh */
    let _ = match Command::new("install_client_key.sh")
    .args(&[credentials.id, filepath])
    .output().await{
        Ok(_)=> return HttpResponse::Ok().finish(),
        Err(_)=> return HttpResponse::BadRequest().finish()
    };
}