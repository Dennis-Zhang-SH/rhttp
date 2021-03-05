use crate::ArrayMap;
use crate::SelfCmp;
use crate::DOUBLE_CRLF;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub path: String,
    pub http_major_version: i32,
    pub http_minor_version: i32,
    pub query_params: ArrayMap<String>,
    pub headers: ArrayMap<String>,
    pub body: String,
}

pub enum RequestReadStatus {
    Started,
    HeaderReaded,
    Finished,
}

impl Request {
    pub async fn new(socket: &mut TcpStream) -> Option<Request> {
        let mut buf = [0; 1024];
        let mut request_buf = Vec::new();
        let mut status: RequestReadStatus = RequestReadStatus::Started;
        let mut rq: Request;
        loop {
            let n = match socket.read(&mut buf).await {
                Ok(n) if n == 0 => break,
                Ok(n) => {
                    request_buf.append(&mut buf[..n].to_vec());
                    status = Request::parse(&mut request_buf, status);
                }
                Err(e) => {
                    eprintln!("failed to read from socket; err = {:?}", e);
                    return None;
                }
            };
        }

        None
    }

    fn parse(request_buf: &mut Vec<u8>, status: RequestReadStatus) -> RequestReadStatus {
        match status {
            RequestReadStatus::Started => match request_buf.contains_part(&*DOUBLE_CRLF) {
                Some((i, j)) => {
                    RequestReadStatus::HeaderReaded
                },
                None => status,
            },
            _ => {
                status
            }
        }
    }

    pub fn method(&self) -> &String {
        &self.method
    }

    pub fn full_url(&self) -> &String {
        &self.url
    }

    pub fn query_params(&self) -> &ArrayMap<String> {
        &self.query_params
    }

    pub fn query(&self, k: impl Into<String>) -> Option<&String> {
        self.query_params.get(k.into())
    }

    // async fn read_body(&mut self, socket: &mut TcpStream, body: &str) {
    //     match self.headers.get("Connection") {
    //         Some(c) if c[0] != "keep-alive" => {
    //             self.body += body;
    //             return;
    //         }
    //         _ => {
    //             match self.headers.get("Content-Length") {
    //                 Some(length) => {
    //                     let length = &length[0];
    //                     match length.parse::<usize>() {
    //                         Ok(length) => {
    //                             if length != body.len() {
    //                                 let mut buf = Vec::new();//with_capacity(length - body.len());
    //                                 buf.resize_with(length - body.len(), Default::default);
    //                                 let _ = socket.read(&mut buf).await;
    //                                 self.body += body;
    //                                 self.body += &String::from_utf8_lossy(&buf);
    //                             } else {
    //                                 self.body += body;
    //                             }
    //                         }
    //                         Err(_) => return,
    //                     }
    //                 }
    //                 _ => return,
    //             }
    //             match self.headers.get("Transfer-Encoding") {
    //                 Some(t) if t[0] == "chunked" => {
    //                     // todo: chucked transfer
    //                     // let chunked_vec: Vec<_> = body.split("\r\n").collect();
    //                 }
    //                 _ => return,
    //             }
    //         }
    //     }
    // }

    // async fn _read_chunked_body(&self) {}

    // pub fn body(&self) -> &String {
    //     &self.body
    // }
}
