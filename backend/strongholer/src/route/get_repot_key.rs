use actix_web::{HttpRequest, Result, http::header, post, web};
use actix_files::NamedFile;
use crate::error::APIError;
use crate::authentification::auth::Auth;
const CLIENT_DIRECTORY: &str = "/srv/repos";

#[post("/get_repot_key")]
async fn get_repot_key(req: HttpRequest, auth: web::Data<Auth>) -> Result<NamedFile, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let (_, (_, credentials)) = auth.validation(cookie.value().to_string());
    let Some(credentials) = credentials else {
        return Err(APIError::NoAuthAppData)
    };
    let filepath = format!("{}/{}/bootstrap/{}.gpg", CLIENT_DIRECTORY, credentials.id,credentials.id);
    match NamedFile::open_async(filepath).await {
        Ok(file)=>{
            let file = file.set_content_disposition(header::ContentDisposition{
                disposition: header::DispositionType::Attachment,
                parameters: vec![header::DispositionParam::Filename(format!("{}.gpg",credentials.id))]
            });
            return Ok(file)
        },
        Err(e)=> {
            println!("User : {}, Erreur{}", credentials.id, e.to_string());
            return Err(APIError::NoFile)
        }
    };
    
    
}