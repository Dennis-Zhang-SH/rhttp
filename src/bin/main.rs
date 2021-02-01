use rhttp::{App, Response};

#[tokio::main]
async fn main() {
    let _ = App::new()
        .register(
            "/",
            Box::new(|_| {
                let mut rs = Response::new(200).unwrap();
                rs.set_header((
                    "Content-Type".to_string(),
                    vec!["text/plain; charset=utf-8".to_string()],
                ));
                rs.set_body(String::from("hello, world"));
                rs
            }),
        )
        .run("127.0.0.1:8080")
        .await;
}
