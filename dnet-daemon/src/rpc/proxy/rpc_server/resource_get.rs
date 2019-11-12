use crate::settings::get_settings;

use super::actix_web::{Error, HttpResponse};
use super::futures::Future;
use crate::info::get_info;

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    VERSION: String,
}

pub fn version() -> impl Future<Item = HttpResponse, Error = Error> {
    let response = Version {
        VERSION: "v1.0.0".to_owned(),
    };

    futures::future::result::<HttpResponse, Error>(
        Ok(HttpResponse::Ok().json(response)))
}

#[derive(Debug, Serialize, Deserialize)]
struct Runtime {
    lastRuntime: String,
    tincLastRuntime: String,
}

pub fn runtime() -> impl Future<Item = HttpResponse, Error = Error> {
    let tincLastRuntime = get_info().try_lock()
        .map(|info|
            info.tinc_info.last_runtime.clone().unwrap_or("None".to_owned()))
        .unwrap_or("Inner Error, please wait.".to_owned());
    let last_runtime =  get_settings().last_runtime.clone();

    let response = Runtime {
        lastRuntime: last_runtime,
        tincLastRuntime,
    };
    futures::future::result::<HttpResponse, Error>(
        Ok(HttpResponse::Ok().json(response)))
}


//fn dump_vlan() -> impl Future<Item = HttpResponse, Error = Error> {
//    let pid_file = get_settings().tinc.home_path.clone() + PID_FILENAME;
//    if let Ok(mut tinc_stream) = TincStream::new(&pid_file) {
//        if let Ok(team_info) = tinc_stream.dump_group() {
//            return Ok(HttpResponse::Ok().json(team_info));
//        }
//    }
//    let response = Response::internal_error();
//    Ok(HttpResponse::Ok().json(response)) // <- send response
//}
//
//fn dump_connections() -> impl Future<Item = HttpResponse, Error = Error> {
//    let pid_file = get_settings().tinc.home_path.clone() + PID_FILENAME;
//    if let Ok(mut tinc_stream) = TincStream::new(&pid_file) {
//        if let Ok(connections) = tinc_stream.dump_connections_parse() {
//            return Ok(HttpResponse::Ok().json(connections));
//        }
//    }
//    let response = Respons::internal_error();
//    Ok(HttpResponse::Ok().json(response)) // <- send response
//}