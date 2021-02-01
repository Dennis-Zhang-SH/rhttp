use http::{Method, StatusCode, Version};
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

type HandleFunction = Box<dyn (Fn(&Request) -> Response) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub path: String,
    pub http_major_version: i32,
    pub http_minor_version: i32,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, Vec<String>>,
    pub body: String,
}

#[derive(Debug)]
pub struct Response {
    pub http_status: http::StatusCode,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Option<String>,
}

impl Request {
    pub fn method(&self) -> &String {
        &self.method
    }

    pub fn full_url(&self) -> &String {
        &self.url
    }

    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }

    pub fn query(&self, k: &str) -> Option<&String> {
        self.query_params.get(k)
    }

    async fn read_body(&mut self, socket: &mut TcpStream, body: &str) {
        match self.headers.get("Connection") {
            Some(c) if c[0] != "keep-alive" => {
                self.body += body;
                return;
            }
            _ => {
                match self.headers.get("Content-Length") {
                    Some(length) => {
                        let length = &length[0];
                        match length.parse::<usize>() {
                            Ok(length) => {
                                if length != body.len() {
                                    let mut buf = Vec::with_capacity(length - body.len());
                                    let _ = socket.read(&mut buf).await;
                                    self.body += body;
                                    self.body += &String::from_utf8_lossy(&buf);
                                } else {
                                    self.body += body;
                                }
                            }
                            Err(_) => return,
                        }
                    }
                    _ => return,
                }
                match self.headers.get("Transfer-Encoding") {
                    Some(t) if t[0] == "chunked" => {
                        // todo: chucked transfer
                        // let chunked_vec: Vec<_> = body.split("\r\n").collect();
                    }
                    _ => return,
                }
            }
        }
    }

    async fn _read_chunked_body(&self) {}

    pub fn body(&self) -> &String {
        &self.body
    }
}

impl Response {
    pub fn new(status: u16) -> Result<Self, http::status::InvalidStatusCode> {
        Ok(Response {
            http_status: StatusCode::from_u16(status)?,
            headers: HashMap::new(),
            body: None,
        })
    }

    pub fn set_status(&mut self, status: u16) -> Result<(), http::status::InvalidStatusCode> {
        self.http_status = StatusCode::from_u16(status)?;
        Ok(())
    }

    pub fn set_header(&mut self, header: (String, Vec<String>)) {
        self.headers.insert(header.0, header.1);
    }

    pub fn set_body(&mut self, body: String) {
        self.body = Some(body);
    }

    pub fn append_body(&mut self, body: String) {
        match self.body.as_mut() {
            Some(b) => b.push_str(body.as_str()),
            None => self.set_body(body),
        }
    }

    pub async fn response_to(mut self, s: &mut TcpStream) {
        self.headers
            .insert("Connection".to_string(), vec!["keep-alive".to_string()]);

        match self.body {
            Some(ref body) => {
                self.headers
                    .insert("Content-Length".to_string(), vec![body.len().to_string()]);
            }
            None => {
                self.headers
                    .insert("Content-Length".to_string(), vec!["0".to_string()]);
            }
        }
        let mut header_string = String::new();
        for header in self.headers.iter() {
            header_string += format!("{}: {}\r\n", header.0, header.1.join(",")).as_str();
        }
        let response = format!(
            "{:?} {}\r\n{}\r\n{}",
            Version::HTTP_11,
            self.http_status,
            header_string,
            self.body.unwrap_or("".to_string())
        );
        let _ = s.write_all(response.as_bytes()).await;
    }
}

pub struct App {
    pub handle_functions: HashMap<&'static str, HandleFunction>,
}

impl App {
    pub fn new() -> Self {
        App {
            handle_functions: HashMap::new(),
        }
    }

    pub fn register(mut self, path: &'static str, f: HandleFunction) -> Self {
        self.handle_functions.insert(path, f);
        self
    }

    pub async fn run(self, addr: &str) -> Result<(), Box<dyn Error>> {
        let app = Arc::new(self);
        let listener = TcpListener::bind(addr).await?;
        loop {
            let (socket, _) = listener.accept().await?;
            tokio::spawn(handle_connection(app.clone(), socket));
        }
    }
}

async fn handle_connection(app: Arc<App>, mut socket: TcpStream) {
    let mut buf = [0; 1024];
    let mut request_string = String::new();
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(n) if n == 0 => return,
            Ok(n) => n,
            Err(e) => {
                let _ = socket
                    .write_all(
                        format!(
                            "{:?} {}\r\nContent-Length:21\r\nContent-Type: text/plain\r\n\r\n{}",
                            Version::HTTP_11,
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Internal server error",
                        )
                        .as_bytes(),
                    )
                    .await;
                eprintln!("failed to read from socket; err = {:?}", e);
                return;
            }
        };
        let buf_string = &String::from_utf8_lossy(&buf[0..n]);
        request_string += buf_string;
        let request_vec;
        if request_string.contains("\r\n\r\n") {
            request_vec = request_string.split("\r\n\r\n").collect::<Vec<_>>();
        } else if request_string.contains("\n\n") {
            request_vec = request_string.split("\n\n").collect::<Vec<_>>();
        } else {
            continue;
        }
        let mut request_header_string = request_vec[0].to_string();
        let mut request = match parse_request_header(&mut request_header_string) {
            Ok(h) => h,
            Err(e) => {
                let _ = socket
                    .write_all(
                        format!(
                            "{:?} {}\r\nContent-Length:{}\r\nContent-Type: text/plain\r\n\r\n{}",
                            Version::HTTP_11,
                            StatusCode::BAD_REQUEST,
                            e.len(),
                            e
                        )
                        .as_bytes(),
                    )
                    .await;
                return;
            }
        };
        request.read_body(&mut socket, request_vec[1]).await;

        match app.handle_functions.get(&request.path.as_str()) {
            Some(f) => {
                let response = f(&request);
                response.response_to(&mut socket).await;
            }
            _ => {
                let _ = socket
                    .write_all(format!(
                        "{:?} {}\r\nContent-Length:14\r\nContent-Type: text/plain\r\n\r\npage not found",
                        Version::HTTP_11,
                        StatusCode::NOT_FOUND
                    ).as_bytes())
                    .await;
            }
        }
        match request.headers.get("Connection") {
            Some(v) if v[0] == "keep-alive" => {
                request_string = "".to_string();
                continue;
            }
            _ => return,
        }
    }
}
fn parse_request_header(requset_header: &mut String) -> Result<Request, &str> {
    let mut request_header_vec = requset_header.split("\n").collect::<Vec<_>>();
    let basic_infos = request_header_vec
        .swap_remove(0)
        .split(" ")
        .map(|x| x.trim())
        .collect::<Vec<_>>();
    if basic_infos.len() < 3 {
        return Err("Invalid request headers");
    }

    let method = match Method::from_str(&basic_infos[0]) {
        Ok(m) => m.to_string(),
        _ => return Err("Unknow method"),
    };

    let mut query_params = HashMap::new();
    let url = basic_infos[1].split("?").collect::<Vec<_>>();
    let path = url[0].to_owned();
    if url.len() > 1 {
        let query_params_vec = url[1].split("&").collect::<Vec<_>>();
        for query_param in query_params_vec {
            let query_param_vec = query_param.split("=").map(|x| x.trim()).collect::<Vec<_>>();
            if query_param_vec.len() == 2 {
                query_params.insert(
                    query_param_vec[0].to_string(),
                    query_param_vec[1].to_string(),
                );
            }
        }
    }

    if !basic_infos[2].starts_with("HTTP") {
        return Err("Unknown protocol");
    }
    let http_protocol = basic_infos[2].split("/").collect::<Vec<_>>();
    if http_protocol.len() < 2 {
        return Err("Invalid http version info");
    }
    let http_version_info = http_protocol[1].split(".").collect::<Vec<_>>();
    if http_version_info.len() < 2 {
        return Err("Invalid http version info");
    }
    let http_major_version: i32 = http_version_info[0].parse().unwrap();
    let http_minor_version: i32 = http_version_info[1].parse().unwrap();
    if http_major_version > 1 {
        return Err("Feature not supported");
    }

    let mut headers = HashMap::new();
    for header in request_header_vec {
        let mut header_vec = header.split(":").map(|x| x.trim()).collect::<Vec<_>>();
        if header_vec.len() >= 2 {
            headers.insert(
                header_vec.swap_remove(0).to_string(),
                header_vec
                    .join(":")
                    .split(",")
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
            );
        }
    }
    if headers.get("Host").is_none() {
        return Err("Missing host in header");
    }

    Ok(Request {
        method,
        url: basic_infos[1].to_string(),
        path,
        http_major_version,
        http_minor_version,
        headers,
        body: String::new(),
        query_params,
    })
}
