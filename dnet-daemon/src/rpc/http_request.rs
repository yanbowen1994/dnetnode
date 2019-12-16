use std::str::FromStr;
use std::time::Duration;
use std::thread::sleep;

use reqwest;

use crate::settings::get_settings;
use crate::info::get_info;

use super::error::*;

const PAGESIZE: usize = 10;

pub fn post(url: &str, data: &str) -> Result<serde_json::Value> {
    let res = loop_post(url, data)?;
    http_error(res)
}

fn loop_post(url: &str, data: &str) -> Result<reqwest::Response> {
    let mut wait_sec = 0;
    loop {
        match url_post(&url, &data) {
            Ok(x) => return Ok(x),
            Err(e) => {
                error!("{:?}", e);
                sleep(std::time::Duration::from_secs(wait_sec));
                if wait_sec < 5 {
                    wait_sec += 1;
                } else {
                    return Err(e);
                }
                continue;
            }
        };
    };
}

fn url_post(url: &str, data: &str)
            -> Result<reqwest::Response> {
    let request_builder = build_request_client(reqwest::Method::POST, url)?;
    let res = request_builder
        .body(data.to_string())
        .send()
        .map_err(Error::Reqwest)?;
    Ok(res)
}

pub fn get_mutipage(url: &str) -> Result<Vec<serde_json::Value>> {
    let mut output = vec![];
    let mut page = 1;
    let is_have_other_param = url.contains("?");
    loop {
        let page_url = if is_have_other_param {
            format!("{}&pageNum={}&pageSize={}", url, page, PAGESIZE)
        }
        else {
            format!("{}?pageNum={}&pageSize={}", url, page, PAGESIZE)
        };
        let res = get(&page_url)?
            .get("records")
            .ok_or(Error::ResponseParse(url.to_string() + "Not Found records."))?
            .to_owned();
        let mut res = res.as_array()
            .ok_or(Error::ResponseParse(url.to_string() + " Can not parse to array."))
            .map_err(|err| {
                error!("get_mutipage {:?}", err);
                err
            })?
            .to_owned();
        if res.len() < PAGESIZE {
            output.append(res.as_mut());
            break
        }
        else {
            output.append(res.as_mut());
            page += 1;
        }
    }
    Ok(output)
}

pub fn get(url: &str) -> Result<serde_json::Value> {
    let res = loop_get(url)?;
    http_error(res)
}

fn loop_get(url: &str)  -> Result<reqwest::Response> {
    let mut wait_sec = 0;
    loop {
        let _res = match url_get(url) {
            Ok(x) => return Ok(x),
            Err(e) => {
                error!("get - response {:?}", e);
                sleep(std::time::Duration::from_secs(wait_sec));
                if wait_sec < 5 {
                    wait_sec += 1;
                }
                else {
                    return Err(e)
                }
                continue;
            }
        };
    }
}

fn url_get(url: &str) -> Result<reqwest::Response> {
    let request_builder = build_request_client(reqwest::Method::GET, url)?;
    let res = request_builder
        .send()
        .map_err(Error::Reqwest)?;
    Ok(res)
}

fn http_error(mut res: reqwest::Response) -> Result<serde_json::Value> {
    if res.status().as_u16() == 200 {
        let res_data = res.text().map_err(Error::Reqwest)?;
        let res_json = serde_json::from_str::<serde_json::Value>(&res_data)
            .map_err(|_|Error::ResponseParse(res_data.to_string()))?;
        let code = res_json.get("code")
            .and_then(|code| {
                code.as_i64()
            })
            .ok_or(Error::ResponseParse(res_json.to_string()))?;

        if code == 200 {
            let result = res_json.get("result")
                .and_then(|value| Some((*value).clone()))
                .unwrap_or(serde_json::Value::Null);
            Ok(result)
        }
        else {
            Err(Error::http(code as i32))
        }
    }
    else {
        Err(Error::http(res.status().as_u16() as i32))
    }
}

fn build_request_client(method: reqwest::Method, url: &str) -> Result<reqwest::RequestBuilder> {
    let token = get_info().lock().unwrap().node.token.clone();
    let client_build = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .http1_title_case_headers()
        .gzip(false);

    let client = if get_settings().common.accept_conductor_invalid_certs {
        client_build
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(Error::Reqwest)
    }
    else {
        client_build
            .build()
            .map_err(Error::Reqwest)
    }?;

    let request_builder = client
        .request(method,
             reqwest::Url::from_str(url)
                 .map_err(|_|Error::UrlParseError)?)
        .header(reqwest::header::CONTENT_TYPE,
                " application/json;charset=UTF-8")
        .header("x-access-token", token)
        .header(reqwest::header::USER_AGENT, "");
    Ok(request_builder)
}