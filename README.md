# RHTTP is a toy HTTP framework
it's more a rust learning project than a production HTTP framework

## RHTTP is NOT very fast
it's literally written in bad codes, e.g. allocated too much.

## RHTTP is simple enough for beginners to try
**usage**:

```rust
use rhttp::{App, Response};

#[tokio::main]
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
        .register(
            "/echopost",
            Box::new(|rq| match rq.method().as_str() {
                "POST" => {
                    // get the body of a request
                    let body = rq.body().clone();
                    let mut rs = Response::new(200).unwrap();
                    rs.set_body(body);
                    rs
                }
                _ => return Response::new(403).unwrap(),
            }),
        )
        .run("127.0.0.1:8080")
        .await;
}
```

in the above example, matching "/" in the url would return a response which header contains a "Content-Type" and body contains a "hello, world".