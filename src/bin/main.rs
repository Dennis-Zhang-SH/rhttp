// use rhttp::{App, Response};

// #[tokio::main]
// async fn main() {
//     let _ = App::new()
//         .register(
//             "/",
//             Box::new(|_| {
//                 let mut rs = Response::new(200).unwrap();
//                 rs.set_header((
//                     "Content-Type".to_string(),
//                     vec!["text/plain; charset=utf-8".to_string()],
//                 ));
//                 rs.set_body(String::from("hello, world"));
//                 rs
//             }),
//         )
//         .register(
//             "/echopost",
//             Box::new(|rq| match rq.method().as_str() {
//                 "POST" => {
//                     // get the body of a request
//                     let body = rq.body().clone();
//                     let mut rs = Response::new(200).unwrap();
//                     rs.set_body(body);
//                     rs
//                 }
//                 _ => return Response::new(403).unwrap(),
//             }),
//         )
//         .run("127.0.0.1:8080")
//         .await;
// }

fn main() {
    let a = "\r\n\r\n".as_bytes().to_vec();
    println!("{:?}", a)
}
