use actix_web::{post, HttpResponse};
use async_std::fs;
use async_std::prelude::*;
use actix_multipart::Multipart;

const MAX_FILE_SIZE_SSH_KEY: usize = 50 * 1024 * 1024;

#[post("sendsshkey")]
async fn send_ssh_key(mut payload: Multipart)->HttpResponse{
     while let Some(field) = payload.next().await {
        let mut field = field.expect("field invalide");
        let filepath= "ssh_key".to_string();
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
    HttpResponse::Ok().finish()
}