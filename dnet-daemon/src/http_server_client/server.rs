#![allow(unused_variables)]
#![allow(non_snake_case)]
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

use crate::info::Info;
use crate::tinc_manager::TincOperator;
use crate::daemon::DaemonEvent;
use crate::settings::get_settings;
use reqwest::header::HeaderValue;

#[derive(Clone)]
struct AppState {
    info: Arc<Mutex<Info>>,
    tinc: Arc<Mutex<TincOperator>>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct KeyReport {
    mac:            String,
    vip:            String,
    pubKey:         String,
    proxypubKey:    String,
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
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct CheckPubkey {
    vip:            String,
    pubKey:         String,
    proxypubKey:    String,
}

#[derive(Debug, Serialize, Deserialize)]
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

    fn not_found(msg: &str) -> Self {
        if msg == "" {
            return Response {
                code: 404,
                data: None,
                msg: Some("Not Found".to_string()),
            };
        }
        else {
            return Response {
                code: 404,
                data: None,
                msg: Some(msg.to_string()),
            };
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug,Serialize, Deserialize)]
struct VirtualIp{
    vip: String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

// if check ok return null else return Response of error.
fn check_apikey(info_arc: Arc<Mutex<Info>>, apikey: Option<&HeaderValue>)
                -> Option<Response> {
    let uid;
    {
        let info = info_arc.lock().unwrap();
        uid = info.proxy_info.uid.clone();
    }

    if let Some(client_apikey) = apikey {
        debug!("http_report_key - response apikey: {:?}", client_apikey);
        if let Ok(client_apikey) = client_apikey.to_str() {
            if client_apikey == &uid {
                return None;
            }
            else {
                error!("http_client - response api key authentication failure");
                let response = Response {
                    code: 401,
                    data: None,
                    msg: Some("Apikey invalid".to_owned()),
                };
                return Some(response);
            }
        }
    }
    error!("http_client - response no api key");

    let response = Response {
        code: 404,
        data: None,
        msg:  Some("No Apikey".to_owned()),
    };
    return Some(response);
}

/// req: http请求
///     req,state() return AppState
///     req.payload() return  tokio 异步操作
///         add_then()中，为body解析方法，可以为闭包或函数
///             HttpResponse::Ok().json(response) 以json格式返回struct Response
fn report_key(req: HttpRequest<AppState>) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();

    let response = check_apikey(
        info_arc.clone(),
        req.headers().get("Apikey"));

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
            if let Some(response) = response {
                return Ok(HttpResponse::Ok().json(response));
            }

            let by = &body.to_vec()[..];
            let req_str = String::from_utf8_lossy(by);

            debug!("http_report_key - response data : {:?}",req_str);

            let response;
            match serde_json::from_str(req_str.as_ref()) {
                Ok(key_report) => {
                    let key_report: KeyReport = key_report;
                    debug!("http_report_key - key_report: {:?}",key_report);
                    let operator = TincOperator::new();
                    let _ = operator.set_hosts(false,
                                               key_report.vip.as_str(),
                                               key_report.pubKey.as_str());
                    response = Response::succeed(key_report.to_json())
                },
                Err(e) => {
                    error!("http_report_key - response KeyReport {}", req_str.as_ref());
                    response = Response::not_found(&("KeyReport ".to_owned() + &e.to_string()));
                },
            }

            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

fn check_key(req: HttpRequest<AppState>) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();

    let response = check_apikey(
        info_arc.clone(),
        req.headers().get("Apikey"));

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
            if let Some(response) = response {
                return Ok(HttpResponse::Ok().json(response));
            }

            let by = &body.to_vec()[..];
            let req_str = String::from_utf8_lossy(by);

            debug!("check_key - response data : {:?}",req_str);

            if let Ok(check_pubkey) = serde_json::from_str(req_str.as_ref()) {
                let check_pubkey: CheckPubkey = check_pubkey;
                debug!("http_check_key - check_pubkey: {:?}", check_pubkey);
                let operator = TincOperator::new();
                let filename = operator.get_client_filename_by_virtual_ip(
                    check_pubkey.vip.as_str());
                if let Ok(pubkey) = operator.get_host_pub_key(
                    filename.as_str()) {
                    if pubkey == check_pubkey.pubKey {
                        let response = Response {
                            code: 200,
                            data: Some(pubkey),
                            msg: None,
                        };
                        return Ok(HttpResponse::Ok().json(response)); // <- send response
                    }
                }
            }

            let response = Response::not_found("");
            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

fn get_key(req: HttpRequest<AppState>) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
    let info_arc: Arc<Mutex<Info>> = req.state().info.clone();

    let response = check_apikey(
        info_arc.clone(),
        req.headers().get("Apikey"));

    debug!("http_report_key - response url : {:?}", req.query());
    let query = req.query();
    let vip = query.get("vip").cloned();

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
            // uuid failed return 401 or 404.
            if let Some(response) = response {
                return Ok(HttpResponse::Ok().json(response));
            }

            let response: Response = match vip {
                Some(vip) => {
                    debug!("get_key - response vip : {}", vip);
                    let operator = TincOperator::new();
                    let filename = operator.get_client_filename_by_virtual_ip(&vip);
                    if let Ok(pubkey) = operator.get_host_pub_key(filename.as_str()) {
                        debug!("get_key - response msg : {}",pubkey);
                        let response = Response {
                            code:   200,
                            data:   Some(pubkey),
                            msg:    None,
                        };
                        response
                    }
                    else {
                        Response::not_found("No such host.")
                    }
                },
                None=>{
                    Response::not_found("")
                }
            };

            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    VERSION: String,
}

fn version(req: HttpRequest<AppState>) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
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
            let response = Version {
                VERSION: "v1.0.5.0".to_owned(),
            };
            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

#[derive(Debug, Serialize, Deserialize)]
struct Runtime {
    lastRuntime: String,
}

fn runtime(req: HttpRequest<AppState>) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
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
            let last_runtime = match get_settings().last_runtime.clone() {
                Some(x) => x,
                None => "".to_owned(),
            };
            let response = Runtime {
                lastRuntime: last_runtime,
            };
            Ok(HttpResponse::Ok().json(response)) // <- send response
        })
        .responder()
}

pub fn web_server(info_arc:     Arc<Mutex<Info>>,
                  tinc_arc:     Arc<Mutex<TincOperator>>,
                  daemon_tx:    std::sync::mpsc::Sender<DaemonEvent>,
) {
    let settings = get_settings();
    let local_port = &settings.proxy.local_port;
    // init
    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
    }
    let sys = actix::System::new("webserver");

    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(&settings.proxy.local_https_server_privkey_file, SslFiletype::PEM)
        .map_err(|e|{
            error!("Https web server could not load privkey.\n  path: {}\n  {}",
                   &settings.proxy.local_https_server_privkey_file,
                   e.to_string());
            let _ = daemon_tx.send(DaemonEvent::ShutDown);
            e
        }).unwrap();
    builder.set_certificate_chain_file(&settings.proxy.local_https_server_certificate_file)
        .map_err(|e|{
            error!("Https web server could not load certificate.\n  path: {}\n  {}",
                   &settings.proxy.local_https_server_privkey_file,
                   e.to_string());
            let _ = daemon_tx.send(DaemonEvent::ShutDown);
            e
        }).unwrap();

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

            .resource("/version", |r| {
                r.method(http::Method::GET).with(version)
            })

            .resource("/runtime", |r| {
                r.method(http::Method::GET).with(runtime)
            })
    }).bind_ssl("0.0.0.0:".to_owned() + local_port, builder)
        .unwrap()
        .start();
//    let _ = sys.run();
}