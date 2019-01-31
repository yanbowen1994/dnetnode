#![allow(unused_variables)]
use std::sync::{Arc, Mutex};

extern crate serde_json;
extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate openssl;

use self::actix_web::{
    error, http, middleware, server, App, AsyncResponder, Error, HttpMessage,
    HttpRequest, HttpResponse
};
use self::openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use self::bytes::BytesMut;
use super::futures::{Future, Stream};
use rustc_serialize::json::encode;

use net_tool::get_local_ip;
use domain::Info;
use tinc_manager::Tinc;

#[derive(Clone)]
struct AppState {
    info: Arc<Mutex<Info>>,
    tinc: Arc<Mutex<Tinc>>,
}
#[derive(Clone, Debug, Deserialize, Serialize, RustcDecodable, RustcEncodable)]
pub struct KeyReport {
    mac:            String,
    vip:            String,
    pub_key:        String,
    proxy_pub_key:  String,
}
impl KeyReport {
    pub fn new() -> Self {
        KeyReport{
            mac:                     "".to_string(),
            vip:                     "".to_string(),
            pub_key:                 "".to_string(),
            proxy_pub_key:           "".to_string(),
        }
    }

    fn new_from_info(info :&Info) -> Self {
        KeyReport{
            mac:                     "".to_string(),
            vip:                     info.tinc_info.vip.to_string(),
            pub_key:                 info.tinc_info.pub_key.clone(),
            proxy_pub_key:           "".to_string(),
        }
    }
    fn to_json(&self) -> String {
        return encode(self).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize, RustcDecodable, RustcEncodable)]
struct Response {
    code:   u32,
    msg:    String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

//fn report_key((info_arc, req): (Arc<Mutex<Info>>, &HttpRequest)
//) -> Box<Future<Item = HttpResponse, Error = Error>> {
fn report_key(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();
    let info = info_arc.lock().unwrap();
    let key_report = KeyReport::new_from_info(&info);

    req.payload()
        .from_err()
        .fold(BytesMut::new(), move |mut body, chunk| {
            if (body.len() + chunk.len()) > MAX_SIZE {
                Err(error::ErrorBadRequest("overflow"))
            } else {
                body.extend_from_slice(&chunk);
                Ok(body)
            }
        })
        .and_then(move |body| {
            let by = &body.to_vec()[..];
            let req = String::from_utf8_lossy(by);
            let response = Response {
                code:   200,
                msg:    key_report.to_json(),
            };
            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

fn check_key(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();
    let info = info_arc.lock().unwrap();
    let key_report = KeyReport::new_from_info(&info);

    req.payload()
        .from_err()
        .fold(BytesMut::new(), move |mut body, chunk| {
            if (body.len() + chunk.len()) > MAX_SIZE {
                Err(error::ErrorBadRequest("overflow"))
            } else {
                body.extend_from_slice(&chunk);
                Ok(body)
            }
        })
        .and_then(|body| {
            let by = &body.to_vec()[..];
            let req = String::from_utf8_lossy(by);
            let response = Response {
                code:   200,
                msg:    "".to_string(),
            };
            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

fn get_key(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();
    let msg;
    {
        let info = info_arc.lock().unwrap();
        msg = info.tinc_info.vip.to_string();
    }


    req.payload()
        .from_err()
        .fold(BytesMut::new(), move |mut body, chunk| {
            if (body.len() + chunk.len()) > MAX_SIZE {
                Err(error::ErrorBadRequest("overflow"))
            } else {
                body.extend_from_slice(&chunk);
                Ok(body)
            }
        })
        .and_then(|body| {
            let by = &body.to_vec()[..];
            let req = String::from_utf8_lossy(by);
            let response = Response {
                code:   200,
                msg,
            };
            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

pub fn web_server(info_arc: Arc<Mutex<Info>>, tinc_arc: Arc<Mutex<Tinc>>) {
    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
    }
    let sys = actix::System::new("webserver");

    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    server::new(move || {
        App::with_state(AppState {info: info_arc.clone(), tinc: tinc_arc.clone()})

            .middleware(middleware::Logger::default())

            .resource("/vppn/tinc/api/v2/proxy/keyreport", |r|
                r.method(http::Method::POST).with(report_key)
            )

            .resource("/vppn/tinc/api/v2/proxy/checkpublickey", |r| {
                r.method(http::Method::POST).with(check_key)
            })

            .resource("/vppn/tinc/api/v2/proxy/getpublickey", |r| {
                r.method(http::Method::GET).with(get_key)
            })

    }).bind_ssl(get_local_ip().unwrap().to_string() + ":8443", builder)
        .unwrap()
        .start();
    let _ = sys.run();
}