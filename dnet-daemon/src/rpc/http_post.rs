use std::str::FromStr;
use std::time::Duration;

use reqwest;

pub type Result<T> = std::result::Result<T, reqwest::Error>;

pub fn url_post(url: &str, data: &str, cookie: &str)
                -> Result<reqwest::Response> {
    let res = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .http1_title_case_headers()
        .gzip(false)
        .build()?
        .request(reqwest::Method::POST,
                 reqwest::Url::from_str(url).unwrap())
        .header(reqwest::header::CONTENT_TYPE,
                " application/json;charset=UTF-8")
        .header(reqwest::header::COOKIE,
                cookie)
        .header(reqwest::header::USER_AGENT, "")
        .body(data.to_string())
        .send()?;
    Ok(res)
}