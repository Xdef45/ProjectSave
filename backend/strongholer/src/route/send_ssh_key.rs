use actix_web::{post, HttpResponse, HttpRequest};
use async_std::{fs,prelude::*, process::Command};
use actix_multipart::Multipart;
use crate::authentification::auth::{Auth, Credentials};
const CLIENT_DIRECTORY: &str = "/"; 

const MAX_FILE_SIZE_SSH_KEY: usize = 50 * 1024 * 1024;

#[post("send_ssh_key")]
async fn send_ssh_key(req: HttpRequest, mut payload: Multipart)->HttpResponse{
    /* Extraction du cookie JWT */
    let id: Credentials = match req.cookie("Bearer"){
        Some(cookie) => {
            match Auth.validation(cookie.value().to_string()).await{
                Ok(id) => id,
                Err(_) => return HttpResponse::BadRequest().body("Pas authentifié"),
            }
        }
        None => return HttpResponse::BadRequest().body("Pas authentifié"),
    };
    
    /* Upload du fichier */
    let filepath= format!("{}/ssh-key.pub", CLIENT_DIRECTORY);
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
                   return HttpResponse::Ok().body("fichier tro gros");
            }
            f.write_all(&chunk)
                .await.expect("impossible d'écrire dans le fichier");
        }
    }

    /* Execution du script d'ajout de la clé ssh */
    let _ = Command::new("/usr/local/sbin/install_client_key.sh")
    .args(&[id.id, filepath])
    .output()
    .await.expect("L'installation de la clé ssh client n'a pas fonctionné");
    HttpResponse::Ok().finish()
}