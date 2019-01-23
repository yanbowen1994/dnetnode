use super::serde_json::Value;
use net_tool::url_get;

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
        let json: Value = get_geo_json();
        GeoInfo {
            country_code: json["country_code"].to_string(),
            country_code3: json["country_code3"].to_string(),
            country_name: json["country_name"].to_string(),
            city: json["city"].to_string(),
            region_code: json["region_code"].to_string(),
            region_name: json["region_name"].to_string(),
            postal_code: json["postal_code"].to_string(),
            region: json["region"].to_string(),
            latitude: json["latitude"].to_string(),
            longitude: json["longitude"].to_string(),
            ipaddr: json["ipaddr"].to_string(),
            dma_code: json["dma_code"].to_string(),
            area_code: json["area_code"].to_string(),
        }
    }

    pub fn flush_geo_info(&mut self) {
        let json: Value = get_geo_json();
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
    }
}

fn get_geo_json() -> Value {
    let (res, _) = url_get("http://52.25.79.82:10000/geoip_json.php");
    let json: Value = serde_json::from_str(&res).unwrap();
    return json;
}