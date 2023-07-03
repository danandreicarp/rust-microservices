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
use lazy_static::lazy_static;
use regex::Regex;
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

lazy_static! {
    static ref INDEX_PATH: Regex = Regex::new("^/(index\\.html?)?$").unwrap();
    static ref USER_PATH: Regex = Regex::new("^/user/((?P<user_id>\\d+?)/?)?$").unwrap();
    static ref USERS_PATH: Regex = Regex::new("^/users/?$").unwrap();
    static ref RAND_PATH: Regex = Regex::new("^/rand/?$").unwrap();
}

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
        let method = req.method();
        let path = req.uri().path();
        let mut users = user_db.lock().unwrap();

        if INDEX_PATH.is_match(path) {
            if method == &Method::GET {
                Response::new(INDEX.into())
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }
        } else if RAND_PATH.is_match(path) {
            if method == Method::GET {
                let random_byte = rand::random::<u8>();
                Response::new(Body::from(random_byte.to_string()))
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }
        } else if USERS_PATH.is_match(path) {
            if method == &Method::GET {
                let list = users
                    .iter()
                    .map(|(id, _)| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                Response::new(list.into())
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }
        } else if let Some(cap) = USER_PATH.captures(path) {
            let user_id = cap
                .name("user_id")
                .and_then(|m| m.as_str().parse::<UserId>().ok().map(|x| x as usize));

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
        } else {
            response_with_code(StatusCode::NOT_FOUND)
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
