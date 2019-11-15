use std::thread::sleep;
use reqwest::Response;
use crate::rpc::http_post::url_post;
use super::rpc_client::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub fn post(url: &str, data: &str, uid: &str) -> Result<Response> {
    let mut wait_sec = 0;
    loop {
        let _res = match url_post(&url, &data, uid) {
            Ok(x) => return Ok(x),
            Err(e) => {
                error!("post - response {:?}", e);
                sleep(std::time::Duration::from_secs(wait_sec));
                if wait_sec < 5 {
                    wait_sec += 1;
                }
                else {
                    return Err(Error::PostError(e));
                }
                continue;
            }
        };
    }
}