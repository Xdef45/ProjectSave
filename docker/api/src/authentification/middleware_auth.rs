use actix_web::{Error, HttpResponse, body::BoxBody, cookie::{Cookie, time::Duration}, dev::{ServiceRequest, ServiceResponse}, middleware::Next, web};
use crate::{authentification::auth::{Auth, BearerState}, error::APIError};

pub async fn authentification_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    // Les route signin et siginup sont exempté
    let path = req.path().to_string();
    if path == "/api/signin".to_string() || path == "/api/signup".to_string() {
        return Ok(next.call(req).await?.map_into_boxed_body())
    }

    // Récupération de auth
    let Some(auth) = req.app_data::<web::Data<Auth>>() else{
        return Ok(req.into_response(HttpResponse::BadRequest().body("501")))
    };

    // Vérification de la présence du cookie Bearer
    let Some(cookie) = req.cookie("Bearer") else{
        return Ok(req.into_response(HttpResponse::BadRequest().body("502")))
    };

    // Vérification de l'authentification
    let (bearer_state, (result, _)) = match auth.validation(cookie.value().to_string()){
        Ok(res)=>res,
        Err(e)=>{
            if e == APIError::Expired{
                // Si expirer suppression du cookie
                let cookie = Cookie::build("Bearer", "")
                        .path("/")
                        .secure(true)
                        .max_age(Duration::milliseconds(0))
                        .http_only(true)
                        .finish();
                return Ok(req.into_response(HttpResponse::Ok()
                .cookie(cookie)
                .body("503")))
            }else{
                return Ok(req.into_response(HttpResponse::BadRequest().body("Erreur validation middleware")))
            }   
        }
    };
    
    // Lancement du service
    let mut res = next.call(req).await?;

    if bearer_state == BearerState::Valid {
        return Ok(res.map_into_boxed_body())
    } else {
        let Some(result) = result else{
            println!("le cookie doit être refresh mais n'est pas donnée");
            return Ok(res.map_into_boxed_body())
        };
        let cookie = Cookie::build("Bearer", result)
                .path("/")
                .secure(true)
                .http_only(true)
                .finish();

        let resc: &mut HttpResponse= res.response_mut();
        resc.add_cookie(&cookie)?;
        return Ok(res.map_into_boxed_body())
    }
}
