use actix_web::{error,HttpResponse};
use derive_more::derive::{Display, Error};


#[derive(Debug, Display, Error)]
pub enum FileError{
    NoFile,
    NoCookieBearer,
    NoAuthAppData
}

impl error::ResponseError for FileError{
    fn error_response(&self)->HttpResponse{
        let response = match *self{
            _=>"0",
            FileError::NoFile=>"1",
            FileError::NoCookieBearer=>"101",
            FileError::NoAuthAppData=>"102"
        };
        HttpResponse::BadRequest().body(response)
    }
}
