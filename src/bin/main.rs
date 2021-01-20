use rhttp::{App, Response};

#[tokio::main]
async fn main() {
    let _ = App::new().register("/".to_string(),Box::new(|_| {
        let mut rs = Response::new(200).unwrap();
        rs.set_body(String::from("hello, world"));
        rs
    })).run("127.0.0.1:8080").await;
}
