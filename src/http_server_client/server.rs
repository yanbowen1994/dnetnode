#![allow(unused_variables)]
#![allow(dead_code)]
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
use rustc_serialize::json::{encode, decode};

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
    pubKey:        String,
    proxypubKey:  String,
}
impl KeyReport {
    pub fn new() -> Self {
        KeyReport{
            mac:                     "".to_string(),
            vip:                     "".to_string(),
            pubKey:                 "".to_string(),
            proxypubKey:           "".to_string(),
        }
    }

    fn new_from_info(info :&Info) -> Self {
        KeyReport{
            mac:                     "".to_string(),
            vip:                     info.tinc_info.vip.to_string(),
            pubKey:                 info.tinc_info.pub_key.clone(),
            proxypubKey:           "".to_string(),
        }
    }
    fn to_json(&self) -> String {
        return encode(self).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize, RustcDecodable, RustcEncodable)]
struct Response {
    code:   u32,
    data:   Option<String>,
    msg:    Option<String>,
}
impl Response {
    fn succeed(msg: String) -> Self {
        Response {
            code:  200,
            data: None,
            msg: Some(msg),
        }
    }

    fn uid_failed() -> Self {
        Response {
            code:  401,
            data: None,
            msg:   Some("No authentication or authentication failure".to_string()),
        }
    }

    fn not_found() -> Self {
        Response {
            code:  404,
            data: None,
            msg:   Some("Not Found".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, RustcDecodable, RustcEncodable)]
struct Request {
    uid:    String,
}
impl Request {
    fn uid_failed() -> Self {
        Request {
            uid: "".to_string(),
        }
    }
}

#[derive(Debug,Serialize, Deserialize, RustcDecodable, RustcEncodable)]
struct VirtualIp{
    vip: String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// req: http请求
///     req,state() return AppState
///     req.payload() return  tokio 异步操作
///         add_then()中，为body解析方法，可以为闭包或函数
///             HttpResponse::Ok().json(response) 以json格式返回struct Response
fn report_key(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();
    let info = info_arc.lock().unwrap().clone();

    debug!("http_report_key - response url : {:?}",req.uri());

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
            let req_str = String::from_utf8_lossy(by).replace("\n","");

            debug!("http_report_key - response data : {:?}",req_str);

            let response:Response  = match req.headers().get("apikey"){
                Some(apikey) => {

                    debug!("http_report_key - response apikey: {:?}",apikey);

                    let response;
                    if apikey.to_str().unwrap() == info.proxy_info.uid {
                        let key_report:KeyReport = decode(req_str.as_str()).unwrap();
                        debug!("http_report_key - key_report: {:?}",key_report);
                        let operator = Tinc::new("/root/tinc".to_string(),"hosts".to_string());
                        let filename = operator.get_filename_by_virtual_ip(key_report.vip.as_str());
                        operator.add_hosts(filename.as_str(),key_report.pubKey.as_str());
                        response = Response::succeed(key_report.to_json())
                    } else {
                        response = Response::uid_failed()
                    }
                    response
                }
                None => {

                    debug!("http_report_key - apikey not found");

                    Response::not_found()
                }
            };

            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

fn check_key(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();
    let info = info_arc.lock().unwrap();
    let key_report = KeyReport::new_from_info(&info);

    debug!("http_report_key - response url : {:?}",req.uri());

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
            let req_str = String::from_utf8_lossy(by);


            debug!("check_key - response data : {:?}",req_str);

            let response = Response {
                code:   200,
                data:   None,
                msg:    Some("".to_string()),
            };
            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

fn get_key(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();
//    let msg;
//    {
//        let info = info_arc.lock().unwrap();
//        msg = info.tinc_info.vip.to_string();
//    }

    debug!("http_report_key - response url : {:?}",req.uri());

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
            let req_str = String::from_utf8_lossy(by);

            debug!("get_key - response data : {:?}",req_str);

            let virtualIp:Option<VirtualIp> = match decode(&req_str){
                Ok(virtualip) => {
                    Some(virtualip)
                },
                Err(e) =>{
                    error!("get_key - resolve json : {:?}",e);
                    None
                }
            };

            let response:Response = match virtualIp{
                Some(virtualip) =>{
                    debug!("get_key - response vip : {}",virtualip.vip);
                    let operator = Tinc::new("/root/tinc".to_string(),"".to_string());
                    let filename = operator.get_filename_by_virtual_ip(virtualip.vip.as_str());
                    let pubkey = operator.get_host_pub_key(filename.as_str());
                    debug!("get_key - response msg : {}",pubkey);
                    let response = Response {
                        code:   200,
                        data: Some(pubkey),
                        msg: None,
                    };
                    response
                },
                None=>{
                    Response::not_found()
                }
            };

            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

pub fn web_server(info_arc: Arc<Mutex<Info>>, tinc_arc: Arc<Mutex<Tinc>>) {
    // init
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
        // init， 传入AppState
        // 启动debug模块
        // 设置路径对应模式， 及 对应操作方法句柄
        // 启动https 服务， 设置绑定ip:端口
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
//    let _ = sys.run();
}