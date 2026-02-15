use actix_web::{error,HttpResponse};
use derive_more::derive::{Display, Error};


#[derive(Debug, Display, Error)]
pub enum APIError{
    NoFile,
    NoCookieBearer,
    NoAuthAppData,
    Script,
    ConversionVecToString,
    Ssh,
    Sftp,
    Write

}

impl error::ResponseError for APIError{
    fn error_response(&self)->HttpResponse{
        let response = match *self{
            APIError::NoFile=>"1",

            // Cas Généraux
            APIError::NoCookieBearer=>"101",
            APIError::NoAuthAppData=>"102",
            APIError::Script=>"103",
            APIError::ConversionVecToString=>"104",
            APIError::Ssh=>"105",
            APIError::Sftp=>"106",

            // File
            APIError::Write=>"200"
        };
        HttpResponse::BadRequest().body(response)
    }
}
