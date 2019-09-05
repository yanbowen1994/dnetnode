use std::thread::sleep;
use std::time::Duration;

use super::serde_json::Value;
use common_core::get_settings;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Can not parse json geo data")]
    ParseJosn(#[error(cause)] ::serde_json::Error),
    #[error(display = "Can not parse json geo data")]
    Reqwest(#[error(cause)] ::reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct GeoInfo {
    pub country_code: String,
    pub country_code3: String,
    pub country_name: String,
    pub region_code: String,
    pub region_name: String,
    pub city: String,
    pub postal_code: String,
    pub region: String,
    pub latitude: String,
    pub longitude: String,
    pub ipaddr: String,
    pub dma_code: String,
    pub area_code: String,
//    x: String,
//    y: String,
//    z: String,
}
impl GeoInfo {
    pub fn new() -> Self {
        GeoInfo {
            country_code:  String::new(),
            country_code3: String::new(),
            country_name:  String::new(),
            city:          String::new(),
            region_code:   String::new(),
            region_name:   String::new(),
            postal_code:   String::new(),
            region:        String::new(),
            latitude:      String::new(),
            longitude:     String::new(),
            ipaddr:        String::new(),
            dma_code:      String::new(),
            area_code:     String::new(),
        }
    }

    pub fn load_local(&mut self) -> Result<()> {
        let settings = get_settings();
        let geo_url = &settings.server.geo_url;
        let json = get_geo_json(geo_url)?;
        self.country_code = json["country_code"].to_string().replace("\"", "");
        self.country_code3 = json["country_code3"].to_string().replace("\"", "");
        self.country_name = json["country_name"].to_string().replace("\"", "");
        self.city = json["city"].to_string().replace("\"", "");
        self.region_code = json["region_code"].to_string().replace("\"", "");
        self.region_name = json["region_name"].to_string().replace("\"", "");
        self.postal_code = json["postal_code"].to_string().replace("\"", "");
        self.latitude = json["latitude"].to_string().replace("\"", "");
        self.longitude = json["longitude"].to_string().replace("\"", "");
        self.ipaddr = json["ipaddr"].to_string().replace("\"", "");
        self.dma_code = json["dma_code"].to_string().replace("\"", "");
        self.area_code = json["area_code"].to_string().replace("\"", "");
        Ok(())
    }
}

fn get_geo_json(geo_url: &str) -> Result<Value> {
    loop {
        if let Ok(mut res) = reqwest::get(geo_url) {
            if res.status().as_u16() == 200 {
                let json_res = serde_json::from_str(
                    &res.text().map_err(Error::Reqwest)?
                ).map_err(Error::ParseJosn)?;
                return Ok(json_res);
            }
        };
        sleep(Duration::new(1, 0));
    }
}