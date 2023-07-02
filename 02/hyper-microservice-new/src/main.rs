use std::{
    convert::Infallible,
    fmt::Display,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use slab::Slab;

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

const USER_PATH: &str = "/user/";

type UserId = u64;
struct UserData;

impl Display for UserData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{}")
    }
}

type UserDb = Arc<Mutex<Slab<UserData>>>;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let user_db: UserDb = Arc::new(Mutex::new(Slab::new()));

    // let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    // let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    let builder = Server::bind(&addr);

    let make_service = make_service_fn(|_conn| {
        let user_db = user_db.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                microservice_handler(req, user_db.clone())
            }))
        }
    });

    let server = builder.serve(make_service);

    // let server = server.map_err(drop);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn microservice_handler(
    req: Request<Body>,
    user_db: UserDb,
) -> Result<Response<Body>, Infallible> {
    let response = {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => Response::new(INDEX.into()),
            (method, path) if path.starts_with(USER_PATH) => {
                let user_id = path
                    .trim_start_matches(USER_PATH)
                    .parse::<UserId>()
                    .ok()
                    .map(|x| x as usize);

                let mut users = user_db.lock().unwrap();

                match (method, user_id) {
                    (&Method::POST, None) => {
                        let id = users.insert(UserData);
                        Response::new(id.to_string().into())
                    }
                    (&Method::POST, Some(_)) => response_with_code(StatusCode::BAD_REQUEST),
                    (&Method::GET, Some(id)) => {
                        if let Some(data) = users.get(id) {
                            Response::new(data.to_string().into())
                        } else {
                            response_with_code(StatusCode::NOT_FOUND)
                        }
                    }
                    (&Method::PUT, Some(id)) => {
                        if let Some(user) = users.get_mut(id) {
                            *user = UserData;
                            response_with_code(StatusCode::OK)
                        } else {
                            response_with_code(StatusCode::NOT_FOUND)
                        }
                    }
                    (&Method::DELETE, Some(id)) => {
                        if users.contains(id) {
                            users.remove(id);
                            response_with_code(StatusCode::OK)
                        } else {
                            response_with_code(StatusCode::NOT_FOUND)
                        }
                    }
                    _ => response_with_code(StatusCode::METHOD_NOT_ALLOWED),
                }
            }
            _ => response_with_code(StatusCode::NOT_FOUND),
        }
    };
    Ok(response)
}

fn response_with_code(status_code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap()
}
