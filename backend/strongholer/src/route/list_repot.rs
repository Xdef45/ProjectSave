use actix_web::{post, HttpResponse, HttpRequest, web};
use crate::authentification::auth::Auth;
const CLIENT_DIRECTORY: &str = "/srv/repos"; 
use tokio::{process::Command, fs, io::AsyncWriteExt};
const MAX_FILE_SIZE_SSH_KEY: usize = 50 * 1024 * 1024;

#[post("/list_repot")]
async fn list_repot(req: HttpRequest, auth: web::Data<Auth>)->HttpResponse{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return HttpResponse::Ok().body("Pas de cookie Bearer")
    };

    let (_, (_, credentials)) = auth.validation(cookie.value().to_string());
    let Some(credentials) = credentials else {
        return HttpResponse::Ok().body("Pas de credentials")
    };
        let filepath= format!("{}/{}/bootstrap/ssh-key.pub", CLIENT_DIRECTORY, credentials.id,);
    println!("{}",filepath);
    let _ = match Command::new(format!("borg list --json"))
    .args(&[credentials.id, filepath])
    .output().await{
            Ok(o)=> {
                println!("Erreur : {}\n Sortie : {}", String::from_utf8(o.stderr).expect("msg"), String::from_utf8(o.stdout).expect("msg"));
                return HttpResponse::Ok().finish()
            },
            Err(_)=> {
                println!("L'installation de la clé ssh client n'a pas fonctionné");
                return HttpResponse::BadRequest().finish()
            }
        };
}