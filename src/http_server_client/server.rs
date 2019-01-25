#![allow(unused_variables)]
extern crate serde_json;

extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate openssl;

use self::actix_web::{
    error, http, middleware, server, App, AsyncResponder, Error, HttpMessage,
    HttpRequest, HttpResponse, Json,
};
use self::openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use self::bytes::BytesMut;
use super::futures::{Future, Stream};
use json::JsonValue;
use rustc_serialize::json::encode;

#[derive(Debug, Serialize, Deserialize, RustcDecodable, RustcEncodable)]
struct Response {
    code: u32,
    msg: String,
}

#[derive(Debug, Serialize, Deserialize, RustcDecodable, RustcEncodable)]
struct MyObj {
    username: String,
    password: String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// This handler manually load request payload and parse json object
fn index_manual(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    println!("ressss");
    // HttpRequest::payload() is stream of Bytes objects
    req.payload()
        // `Future::from_err` acts like `?` in that it coerces the error type from
        // the future into the final error type
        .from_err()

        // `fold` will asynchronously read each chunk of the request body and
        // call supplied closure, then it resolves to result of closure
        .fold(BytesMut::new(), move |mut body, chunk| {
            // limit max size of in-memory payload
            if (body.len() + chunk.len()) > MAX_SIZE {
                Err(error::ErrorBadRequest("overflow"))
            } else {
                body.extend_from_slice(&chunk);
                Ok(body)
            }
        })
        // `Future::and_then` can be used to merge an asynchronous workflow with a
        // synchronous workflow
        .and_then(|body| {
            let by = &body.to_vec()[..];
            let req = String::from_utf8_lossy(by);
            println!("{}", req);

            let obj = MyObj{
                username: "444444444444".to_string(),
                password: "1".to_string(),
            };
            let response = Response {
                code: 200,
                msg: encode(&obj).unwrap(),
            };
            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

pub fn web_server() {
    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
    }
    let sys = actix::System::new("ws-example");

    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    server::new(|| {
        App::new()
            // enable logger
            .middleware(middleware::Logger::default())

            .resource("/login", |r| {
                r.method(http::Method::POST).f(index_manual)
            })

    }).bind_ssl("127.0.0.1:8443", builder)
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8443");
    let _ = sys.run();
}