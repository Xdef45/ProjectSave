use actix_web::{error,HttpResponse};
use derive_more::derive::{Display, Error};


#[derive(Debug, Display, Error)]
pub enum APIError{
    NoFile,
    NoCookieBearer,
    NoAuthAppData,
    Script,
    ConversionVecToString

}

impl error::ResponseError for APIError{
    fn error_response(&self)->HttpResponse{
        let response = match *self{
            APIError::NoFile=>"1",

            // Cas Généraux
            APIError::NoCookieBearer=>"101",
            APIError::NoAuthAppData=>"102",
            APIError::Script=>"103",
            APIError::ConversionVecToString=>"104"
        };
        HttpResponse::BadRequest().body(response)
    }
}
