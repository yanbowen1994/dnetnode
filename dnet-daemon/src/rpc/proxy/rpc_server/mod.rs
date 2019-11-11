#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(dead_code)]
use std::sync::{Arc, Mutex};

extern crate serde_json;
extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate openssl;
extern crate futures;

use self::actix_web::{client::Client, web, App, HttpServer};
use self::openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use crate::tinc_manager::TincOperator;
use crate::daemon::DaemonEvent;
use crate::settings::get_settings;

mod resource_get;
mod resource_post;
use resource_get::*;
use resource_post::*;
mod types;

// max payload size is 256k

//// if check ok return null else return Response of error.
//fn check_apikey(apikey: Option<&HeaderValue>) -> Option<Response> {
//    let uid;
//    {
//        let info = get_info().lock().unwrap();
//        uid = info.proxy_info.uid.clone();
//    }
//
//    if let Some(client_apikey) = apikey {
//        debug!("http_report_key - response apikey: {:?}", client_apikey);
//        if let Ok(client_apikey) = client_apikey.to_str() {
//            if client_apikey == &uid {
//                return None;
//            }
//            else {
//                error!("http_client - response api key authentication failure");
//                let response = Response {
//                    code: 401,
//                    data: None,
//                    msg: Some("Apikey invalid".to_owned()),
//                };
//                return Some(response);
//            }
//        }
//    }
//    error!("http_client - response no api key");
//
//    let response = Response {
//        code: 404,
//        data: None,
//        msg:  Some("No Apikey".to_owned()),
//    };
//    return Some(response);
//}
//
///// req: http请求
/////     req,state() return AppState
/////     req.payload() return  tokio 异步操作
/////         add_then()中，为body解析方法，可以为闭包或函数
/////             HttpResponse::Ok().json(response) 以json格式返回struct Response
//fn report_key(mut req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
//    let response = check_apikey(req.headers().get("Apikey"));
//    debug!("http_report_key - response url : {:?}",req.uri());
//
//    req.take_payload()
////        .take()
////        .from_err()
//        .fold(BytesMut::new(), move |mut body, chunk| {
//            if (body.len() + chunk.len()) > MAX_SIZE {
//                Err(error::ErrorBadRequest("overflow"))
//            } else {
//                body.extend_from_slice(&chunk);
//                Ok(body)
//            }
//        })
//        .and_then(move |body| {
//            if let Some(response) = response {
//                return Ok(HttpResponse::Ok().json(response));
//            }
//
//            let by = &body.to_vec()[..];
//            let req_str = String::from_utf8_lossy(by);
//
//            debug!("http_report_key - response data : {:?}",req_str);
//
//            let response;
//            match serde_json::from_str(req_str.as_ref()) {
//                Ok(host_up) => {
//                    let host_up: HostUp = host_up;
//                    debug!("http_report_key - host_up: {:?}",host_up);
//                    let operator = TincOperator::new();
//                    let _ = operator.set_hosts(false,
//                                               host_up.vip.as_str(),
//                                               host_up.pubKey.as_str());
//                    response = Response::succeed(host_up.to_json())
//                },
//                Err(e) => {
//                    error!("http_report_key - response HostUp {}", req_str.as_ref());
//                    response = Response::not_found(&("HostUp ".to_owned() + &e.to_string()));
//                },
//            }
//
//            Ok(HttpResponse::Ok().json(response)) // <- send response
//        })
//        .responder()
//}
//
//fn host_up(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
//    let response = check_apikey(req.headers().get("Apikey"));
//    debug!("host_up - response url : {:?}",req.uri());
//
//    req.payload()
//        .from_err()
//        .fold(BytesMut::new(), move |mut body, chunk| {
//            if (body.len() + chunk.len()) > MAX_SIZE {
//                Err(error::ErrorBadRequest("overflow"))
//            } else {
//                body.extend_from_slice(&chunk);
//                Ok(body)
//            }
//        })
//        .and_then(move |body| {
//            if let Some(response) = response {
//                return Ok(HttpResponse::Ok().json(response));
//            }
//
//            let by = &body.to_vec()[..];
//            let req_str = String::from_utf8_lossy(by);
//
//            debug!("host_up - response data : {:?}",req_str);
//
//            let response;
//            match serde_json::from_str(req_str.as_ref()) {
//                Ok(key_report) => {
//                    let key_report: KeyReport = key_report;
//                    debug!("host_up - key_report: {:?}",key_report);
//                    let operator = TincOperator::new();
//                    let _ = operator.set_hosts(false,
//                                               key_report.vip.as_str(),
//                                               key_report.pubKey.as_str());
//                    response = Response::succeed(key_report.to_json())
//                },
//                Err(e) => {
//                    error!("host_up - response KeyReport {}", req_str.as_ref());
//                    response = Response::not_found(&("HostUp ".to_owned() + &e.to_string()));
//                },
//            }
//
//            Ok(HttpResponse::Ok().json(response)) // <- send response
//        })
//        .responder()
//}
//
//fn check_key(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
//    let response = check_apikey(
//        req.headers().get("Apikey"));
//
//    debug!("http_report_key - response url : {:?}",req.uri());
//
//    req.payload()
//        .from_err()
//        .fold(BytesMut::new(), move |mut body, chunk| {
//            if (body.len() + chunk.len()) > MAX_SIZE {
//                Err(error::ErrorBadRequest("overflow"))
//            } else {
//                body.extend_from_slice(&chunk);
//                Ok(body)
//            }
//        })
//        .and_then(|body| {
//            if let Some(response) = response {
//                return Ok(HttpResponse::Ok().json(response));
//            }
//
//            let by = &body.to_vec()[..];
//            let req_str = String::from_utf8_lossy(by);
//
//            debug!("check_key - response data : {:?}",req_str);
//
//            if let Ok(check_pubkey) = serde_json::from_str(req_str.as_ref()) {
//                let check_pubkey: CheckPubkey = check_pubkey;
//                debug!("http_check_key - check_pubkey: {:?}", check_pubkey);
//                let operator = TincOperator::new();
//
//                let filename = operator.get_client_filename_by_virtual_ip(
//                    check_pubkey.vip.as_str());
//                if let Ok(pubkey) = operator.get_host_pub_key(
//                    filename.as_str()) {
//                    if pubkey == check_pubkey.pubKey {
//                        let response = Response {
//                            code: 200,
//                            data: Some(pubkey),
//                            msg: None,
//                        };
//                        return Ok(HttpResponse::Ok().json(response)); // <- send response
//                    }
//                }
//            }
//
//            let response = Response::failed("Host file not found.");
//            Ok(HttpResponse::Ok().json(response)) // <- send response
//        })
//        .responder()
//}
//
//fn get_key(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>>  {
//    let response = check_apikey(
//        req.headers().get("Apikey"));
//
//    debug!("http_report_key - response url : {:?}", req.query());
//    let query = req.query();
//    let vip = query.get("vip").cloned();
//
//    req.payload()
//        .from_err()
//        .fold(BytesMut::new(), move |mut body, chunk| {
//            if (body.len() + chunk.len()) > MAX_SIZE {
//                Err(error::ErrorBadRequest("overflow"))
//            } else {
//                body.extend_from_slice(&chunk);
//                Ok(body)
//            }
//        })
//        .and_then(|body| {
//            // uuid failed return 401 or 404.
//            if let Some(response) = response {
//                return Ok(HttpResponse::Ok().json(response));
//            }
//
//            let response: Response = match vip {
//                Some(vip) => {
//                    debug!("get_key - response vip : {}", vip);
//                    let operator = TincOperator::new();
//                    let filename = operator.get_client_filename_by_virtual_ip(&vip);
//                    if let Ok(pubkey) = operator.get_host_pub_key(filename.as_str()) {
//                        debug!("get_key - response msg : {}",pubkey);
//                        let response = Response {
//                            code:   200,
//                            data:   Some(pubkey),
//                            msg:    None,
//                        };
//                        response
//                    }
//                    else {
//                        Response::not_found("No such host.")
//                    }
//                },
//                None=>{
//                    Response::not_found("")
//                }
//            };
//
//            Ok(HttpResponse::Ok().json(response)) // <- send response
//        })
//        .responder()
//}

pub(super) fn web_server(
                  tinc_arc:            Arc<Mutex<TincOperator>>,
                  daemon_tx:           std::sync::mpsc::Sender<DaemonEvent>,
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

    HttpServer::new(move || {
        // init， 传入AppState
        // 启动debug模块
        // 设置路径对应模式， 及 对应操作方法句柄
        // 启动https 服务， 设置绑定ip:端口
        App::new().data(Client::default())
            .service(
                web::resource("/vppn/tinc/api/v2/proxy/keyreport")
                    .route(web::post().to_async(report_key)))
            
            .service(
                web::resource("/vppn/tinc/api/v2/proxy/host_up")
                    .route(web::post().to_async(report_key)))

            .service(
                web::resource("/vppn/tinc/api/v2/proxy/checkpublickey")
                    .route(web::post().to_async(check_key)))
            
            .service(
                web::resource("/vppn/tinc/api/v2/proxy/getpublickey")
                    .route(web::post().to_async(get_key)))

            .service(
                web::resource("/version")
                    .route(web::get().to_async(version)))

            .service(
                web::resource("/runtime")
                    .route(web::get().to_async(runtime)))
    }).bind_ssl("0.0.0.0:".to_owned() + local_port, builder)
        .unwrap()
        .start();
//    let _ = sys.run();
}