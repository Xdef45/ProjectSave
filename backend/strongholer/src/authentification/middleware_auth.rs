use actix_web::{Error, HttpResponse, body::BoxBody, cookie::{Cookie, time::Duration}, dev::{ServiceRequest, ServiceResponse}, middleware::Next, web};
use crate::authentification::auth::{Auth, BearerState};

pub async fn authentification_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let path = req.path().to_string();
    if path == "/api/signin".to_string() || path == "/api/signup".to_string() {
        return Ok(next.call(req).await?.map_into_boxed_body())
    }
    let auth = match req.app_data::<web::Data<Auth>>(){
        Some(auth) => auth,
        None => return Ok(req.into_response(HttpResponse::Ok().body("Error")))
    };
    // est ce que le cookie existe ?
    let cookie = match req.cookie("Bearer"){
        Some(cookie)=>cookie,
        None => return Ok(req.into_response(HttpResponse::Ok().body("Pas de coofkie Bearer")))
    };
    let (bearer_state, (result, _)) = auth.validation(cookie.value().to_string());
    if bearer_state == BearerState::Valid {
        return Ok(next.call(req).await?.map_into_boxed_body())
    }
    if bearer_state == BearerState::Refresh{
        let mut res = next.call(req).await?;
        let cookie = Cookie::build("Bearer", result.expect(""))
                .path("/")
                .secure(true)
                .http_only(true)
                .finish();
        let resc: &mut HttpResponse= res.response_mut();
        resc.add_cookie(&cookie)?;
        return Ok(res.map_into_boxed_body())
    }
    if bearer_state == BearerState::Expired{
        let cookie = Cookie::build("Bearer", result.expect(""))
                .path("/")
                .secure(true)
                .max_age(Duration::milliseconds(0))
                .http_only(true)
                .finish();
        return Ok(req.into_response(HttpResponse::Ok()
        .cookie(cookie)
        .body("Token expiré")))
    }
    // invoke the wrapped middleware or service
    
    // post-processing

    Ok(next.call(req).await?.map_into_boxed_body())
}
