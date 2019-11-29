use std::str::FromStr;
use std::collections::HashMap;
use std::net::IpAddr;

use reqwest::header::HeaderValue;
use actix_web::{
    web,
    error, Error,
    HttpRequest, HttpResponse
};
use futures::{Future, Stream};
use bytes::BytesMut;
use serde_json::json;
use tinc_plugin::{TincTeam, PID_FILENAME};
use dnet_types::response::Response;

use crate::tinc_manager::TincOperator;
use crate::info::get_info;
use crate::settings::get_settings;
use dnet_types::settings::RunMode;

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


pub fn update_team_info(req: HttpRequest,
                        payload: web::Payload
) -> impl Future<Item = HttpResponse, Error = Error> {
    parse_payload(req, payload, update_team_info_inner)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct KeyReport {
    deviceSerial:   String,
    vip:            String,
    pubKey:         String,
    proxyPubkey:    Option<String>,
}
impl KeyReport {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

pub fn report_key_inner(body: String) -> Result<HttpResponse, Error> {
    debug!("http_report_key - response data : {:?}", body);

    let mut response = Response::success();

    match serde_json::from_str(body.as_str()) {
        Ok(key_report) => {
            let client_vec: Vec<KeyReport> = key_report;
            debug!("http_report_key - key_report: {:?}", client_vec);
            let operator = TincOperator::new();

            for client in client_vec {
                if client.pubKey.len() > 0 {
                    let res = IpAddr::from_str(&client.vip)
                        .ok()
                        .and_then(|vip| {
                            operator.set_hosts(
                                None,
                                vip,
                                client.pubKey.as_str(),
                            ).ok()
                        });

                    if let None = res {
                        response = Response::new_from_code(500);
                        break
                    }
                }
            }
        },
        Err(_) => error!("http_report_key - response KeyReport {}", body.as_str()),
    }
    if response.code == 200 {
        Ok(HttpResponse::Ok().json(response))
    }
    else {
        Ok(HttpResponse::Ok().json(response))
    }
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
                    data: Some(json!(pubkey)),
                    msg:  String::new(),
                };
                return Ok(HttpResponse::Ok().json(response)); // <- send response
            }
        }
    }

    let response = Response::new_from_code(404);
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

    let data = Some(json!(&output));

    let response = Response {
        code:   200,
        data,
        msg:    String::new(),
    };
    Ok(HttpResponse::Ok().json(response)) // <- send response
}

fn update_team_info_inner(body: String) -> Result<HttpResponse, Error> {
    if get_settings().common.mode == RunMode::Center {
        info!("update_team_info - response data : {}",body);

        let mut response = Response::internal_error();

        match serde_json::from_str::<TincTeam>(&body) {
            Ok(tinc_team) => {
                info!("server team change: {:?}", tinc_team);
                    let tinc_pid = get_settings().common.home_path
                        .join("tinc").join(PID_FILENAME)
                        .to_str().unwrap().to_string();
                    match tinc_team.send_to_tinc(&tinc_pid) {
                        Ok(_) => response = Response::success(),
                        Err(failed_team) => {
                            if let Ok(value) = serde_json::to_value(&failed_team) {
                                response = Response::internal_error()
                                    .set_data(Some(value))
                            }
                            else {
                                response = Response::internal_error()
                            }
                        }
                    }
                }
            Err(_) => error!("update_group_info - can't parse: {}", body.as_str()),
        }
        if response.code == 200 {
            return Ok(HttpResponse::Ok().json(response)); // <- send response
        }
        else {
            return Ok(HttpResponse::Ok().json(response));
        }
    }
    else {
        return Ok(HttpResponse::Ok().json(""));
    }
}

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
        uid = info.proxy_info.auth_id.clone().unwrap();
    }

    if let Some(client_apikey) = apikey {
        if let Ok(client_apikey) = client_apikey.to_str() {
            if client_apikey == &uid {
                info!("check_apikey - request apikey: {:?}", client_apikey);
                return None;
            }
            else {
                error!("http_client - response api key authentication failure");
                let response = Response::new_from_code(401);

                return Some(response);
            }
        }
    }
    error!("http_client - response no api key");

    let response = Response {
        code: 404,
        data: None,
        msg:  "No Apikey".to_owned(),
    };
    return Some(response);
}