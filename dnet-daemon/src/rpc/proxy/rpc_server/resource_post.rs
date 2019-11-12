use std::collections::HashMap;

use reqwest::header::HeaderValue;

use actix_web::{
    web,
    error, Error,
    HttpRequest, HttpResponse
};
use futures::{Future, Stream};
use bytes::BytesMut;

use crate::tinc_manager::TincOperator;
use super::types::Response;
use crate::info::get_info;

const MAX_SIZE: usize = 262_144;

pub fn report_key(req: HttpRequest,
                  payload: web::Payload
) -> impl Future<Item = HttpResponse, Error = Error> {
    parse_payload(req, payload, report_key_inner)
}

pub fn check_key(req: HttpRequest,
                 payload: web::Payload
) -> impl Future<Item = HttpResponse, Error = Error> {
    parse_payload(req, payload, check_key_inner)
}

pub fn get_key(req: HttpRequest,
               payload: web::Payload
) -> impl Future<Item = HttpResponse, Error = Error> {
    parse_payload(req, payload, get_key_inner)
}


//pub fn update_team_info(req: HttpRequest,
//                        payload: web::Payload
//) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
//    parse_payload(req, payload, update_team_info_inner)
//}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct Vip {
    mac:            String,
    vip:            String,
    pubKey:         String,
}
impl Vip {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct KeyReport {
    vips: Vec<Vip>,
}
impl KeyReport {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

pub fn report_key_inner(body: String) -> Result<HttpResponse, Error> {

    debug!("http_report_key - response data : {:?}", body);

    let mut response = Response::uid_failed();

    match serde_json::from_str(body.as_str()) {
        Ok(key_report) => {
            let key_report: KeyReport = key_report;
            debug!("http_report_key - key_report: {:?}",key_report);
            let operator = TincOperator::new();

            let mut failed_ip = vec![];
            for client in key_report.clone().vips {
                if let Err(e) = operator.set_hosts(
                    false,
                    client.vip.as_str(),
                    client.pubKey.as_str()
                ) {
                    failed_ip.push(client.vip)
                }
            }
            if failed_ip.len() == 0 {
                response = Response::succeed(key_report.to_json())
            }
            else {
                response = Response::set_host_failed(failed_ip);
            }
        },
        Err(_) => error!("http_report_key - response KeyReport {}", body.as_str()),
    }
    Ok(HttpResponse::Ok().json(response))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct CheckPubkey {
    vip:            String,
    pubKey:         String,
    proxypubKey:    String,
}

fn check_key_inner(body: String) -> Result<HttpResponse, Error> {
    if let Ok(check_pubkey) = serde_json::from_str(body.as_str()) {
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

    let response = Response::not_found();
    Ok(HttpResponse::Ok().json(response)) // <- send response
}

fn get_key_inner(body: String) -> Result<HttpResponse, Error> {
    let vips = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .as_ref()
        .and_then(|value|value.get("vip"))
        .and_then(|value|value.as_array())
        .and_then(|vips|{
            Some(vips.iter()
                .filter_map(|vip|vip.as_str())
                .collect::<Vec<&str>>()
                .iter()
                .map(|vip|(**vip).to_string())
                .collect::<Vec<String>>())
        }).unwrap();

    debug!("get_key - response vips : {:?}", vips);
    let mut output = HashMap::new();

    let operator = TincOperator::new();

    for vip_str in vips {
        let filename;

        if &vip_str != "vpnserver" {
            filename = operator.get_client_filename_by_virtual_ip(&vip_str);
        } else {
            filename = "vpnserver".to_owned();
        }

        if let Ok(pubkey) = operator.get_host_pub_key(filename.as_str()) {
            output.insert(vip_str, pubkey);
        }
    }

    let data = Some(serde_json::to_string(&output).unwrap());

    let response = Response {
        code:   200,
        data,
        msg:    None,
    };
    Ok(HttpResponse::Ok().json(response)) // <- send response
}

//fn update_team_info_inner(body: String) -> Result<HttpResponse, Error> {
//    info!("update_group_info - response data : {}",body);
//
//    let mut response = Response::internal_error();
//
//    match serde_json::from_str(body.as_str()) {
//        Ok(request) => {
//            let request: TeamInfo = request;
//            info!("server team change: {:?}", request);
//
//            let mut info = info.lock().unwrap();
//            let change = info.proxy_info.team_info.increment_modify(&request);
//            let tinc = TincOperator::new();
//            if !tinc.send_node_team_info(&change.0, &change.1) {
//                error!("update_team_info failed.");
//            }
//            response = Response::succeed("".to_owned());
//        },
//        Err(_) => error!("update_group_info - can't parse: {}", body.as_str()),
//    }
//
//    Ok(HttpResponse::Ok().json(response)) // <- send response
//}

fn parse_payload<F>(
    req: HttpRequest,
    payload: web::Payload,
    f: F,
) -> impl Future<Item = HttpResponse, Error = Error>
    where F: 'static + FnOnce(String) -> Result<HttpResponse, Error>
{
    let api_key = req.headers().get("Apikey");
    let response = check_apikey(api_key);

    payload
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
                Ok(HttpResponse::Ok().json(response))
            }
            else {
                let by = &body.to_vec()[..];
                let body = String::from_utf8_lossy(by).replace("\n", "");
                f(body)
            }
        })
}

// if check ok return null else return Response of error.
fn check_apikey(apikey: Option<&HeaderValue>)
                -> Option<Response> {
    let uid;
    {
        let info = get_info().lock().unwrap();
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