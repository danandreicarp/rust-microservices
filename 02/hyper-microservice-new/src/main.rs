use std::{
    convert::Infallible,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};

const INDEX: &'static str = r#"
 <!doctype html>
 <html>
     <head>
         <title>Rust Microservice</title>
     </head>
     <body>
         <h3>Rust Microservice</h3>
     </body>
 </html>
 "#;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    // let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    let builder = Server::bind(&addr);

    let new_service =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(microservice_handler)) });

    let server = builder.serve(new_service);

    // let server = server.map_err(drop);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn microservice_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(INDEX.into())),
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
            return Ok(response);
        }
    }
}
