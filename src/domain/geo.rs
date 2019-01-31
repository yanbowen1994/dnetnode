use super::serde_json::Value;
use net_tool::url_get;
use settings::Settings;

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

    pub fn load_local(&mut self, settings: &Settings) -> bool {
        let geo_url = &settings.server.geo_url;
        if let Some(json) = get_geo_json(geo_url) {
            self.country_code = json["country_code"].to_string();
            self.country_code3 = json["country_code3"].to_string();
            self.country_name = json["country_name"].to_string();
            self.city = json["city"].to_string();
            self.region_code = json["region_code"].to_string();
            self.region_name = json["region_name"].to_string();
            self.postal_code = json["postal_code"].to_string();
            self.latitude = json["latitude"].to_string();
            self.longitude = json["longitude"].to_string();
            self.ipaddr = json["ipaddr"].to_string();
            self.dma_code = json["dma_code"].to_string();
            self.area_code = json["area_code"].to_string();
            return true;
        }
        else {
            return false;
        };

    }
}

fn get_geo_json(geo_url: &str) -> Option<Value> {
    let res = match url_get(geo_url) {
        Ok(res) => res,
        Err(_) => return None,
    };
    if res.code == 200 {
        let json: Value = serde_json::from_str(&res.data).unwrap();
        return Some(json);
    }
    else {
        return None
    }
}