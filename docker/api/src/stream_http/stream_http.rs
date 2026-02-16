use futures_core::stream::Stream;
use tokio::io::{AsyncRead,ReadBuf};
use futures_util::task::{Poll, Context};
use std::pin::Pin;
use bytes::Bytes;
use openssh_sftp_client::file::TokioCompatFile;


pub struct StreamBuffer{
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