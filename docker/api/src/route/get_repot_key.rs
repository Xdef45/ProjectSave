use bytes::Bytes;
use actix_web::HttpResponse;
use actix_web::{HttpRequest, Result, post, web};
use openssh_sftp_client::file::TokioCompatFile;
use crate::error::APIError;
use crate::authentification::auth::Auth;
const CLIENT_DIRECTORY: &str = "/srv/repos";
use futures_core::stream::Stream;
use tokio::io::{AsyncRead,ReadBuf};
use futures_util::task::{Poll, Context};
use std::pin::Pin;

struct StreamBuffer{
    reader: Pin<Box<TokioCompatFile>>,
    buf: [u8; 16*1024]
}

impl StreamBuffer {
    pub fn new(reader: TokioCompatFile )->Self{
        Self{reader:Box::pin(reader), buf: [0u8;16*1024]}
    }
}

impl Stream for StreamBuffer
    {
    type Item = Result<Bytes, std::io::Error>;
    fn poll_next(mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        let mut buffer = ReadBuf::new(&mut this.buf);

        match this.reader.as_mut().poll_read(cx, &mut buffer) {
            Poll::Ready(Ok(())) => {
                let chunk = buffer.filled();
                if chunk.is_empty() {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(Bytes::copy_from_slice(chunk))))
                }
            }
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[post("/get_repot_key")]
async fn get_repot_key(req: HttpRequest, auth: web::Data<Auth>) -> Result<HttpResponse, APIError>{
    /* Extraction du cookie JWT */
    let Some(cookie) = req.cookie("Bearer") else{
        return Err(APIError::NoCookieBearer)
    };

    let (_, (_, credentials)) = auth.validation(cookie.value().to_string());
    let Some(credentials) = credentials else {
        return Err(APIError::NoAuthAppData)
    };
    let filepath = format!("{}/{}/bootstrap/{}.gpg", CLIENT_DIRECTORY, credentials.id,credentials.id);
    let repot_key = match auth.sftp_connexion.open(filepath).await{
        Ok(f)=>f,
        Err(_)=>return Err(APIError::Script)
    };
    let reader = TokioCompatFile::from(repot_key);
    let stream = StreamBuffer::new(reader);
    return Ok(HttpResponse::Ok().streaming(stream))
    
}
