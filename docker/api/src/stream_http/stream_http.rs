use futures_core::{stream::Stream, task::{Poll, Context}};
use tokio::io::{AsyncRead,ReadBuf};
use std::pin::Pin;
use bytes::Bytes;
use openssh_sftp_client::file::TokioCompatFile;


pub struct StreamBuffer{
    reader: Pin<Box<TokioCompatFile>>,
    buf: [u8; 32*1024]
}

impl StreamBuffer {
    pub fn new(reader: TokioCompatFile )->Self{
        Self{reader:Box::pin(reader), buf: [0u8;32*1024]}
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


pub struct StreamBuffer2{
    data: Vec<u8>,
    offset: usize,
    chunk_size: usize,
}

impl StreamBuffer2 {
    pub fn new(reader: Vec<u8>)->Self{
        Self {
            data: reader,
            offset: 0,
            chunk_size: 32 * 1024,
        }
    }
}

impl Stream for StreamBuffer2
    {
    type Item = Result<Bytes, std::io::Error>;
    fn poll_next(mut self: Pin<&mut Self>,
        _: &mut Context<'_>,) -> Poll<Option<Self::Item>> {
        if self.offset >= self.data.len() {
            Poll::Ready(None)
        } else {
            let end = (self.offset + self.chunk_size).min(self.data.len());
            let chunk = Bytes::copy_from_slice(&self.data[self.offset..end]);
            self.offset = end;
            Poll::Ready(Some(Ok(chunk)))
        }
    }
}
