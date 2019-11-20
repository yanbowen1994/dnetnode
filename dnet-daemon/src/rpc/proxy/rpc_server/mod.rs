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
                web::resource("/center/vlantagging/change")
                    .route(web::post().to_async(update_team_info)))

            .service(
                web::resource("/version")
                    .route(web::get().to_async(version)))

            .service(
                web::resource("/runtime")
                    .route(web::get().to_async(runtime)))
    }).bind_ssl("0.0.0.0:".to_owned() + &format!("{}", local_port), builder)
        .unwrap()
        .start();
//    let _ = sys.run();
}