use rhttp::App;

#[tokio::main]
async fn main() {
    let _ = App::new().register("/".to_string(),Box::new(|_, mut rs| {
        rs.set_status(200).unwrap();
        rs.set_body(String::from("hello, world"));
        rs
    })).run("127.0.0.1:8080").await;
}
