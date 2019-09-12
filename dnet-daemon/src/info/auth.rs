use super::Info;

const BACKGROUND_STATUSNOTIFY: &str = "/vppn/api/v2/client/statusNotify";

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    url:            String,
    Apikey:         String,
    proxyIp:        String,
    cookie:         String,
    Authorization:  String,
}
impl AuthInfo {
    #[allow(non_snake_case)]
    pub fn load(server_url: &str, info: &Info) -> Self {
        let url = server_url.to_string() + BACKGROUND_STATUSNOTIFY;
        let Apikey = info.proxy_info.uid.clone();
        let proxyIp = info.proxy_info.proxy_ip.clone();
        let cookie = info.proxy_info.cookie.clone().replace("\r\n", "");
        let Authorization = "test".to_string();
        AuthInfo {
            url,
            Apikey,
            proxyIp,
            cookie,
            Authorization,
        }
    }
    pub fn to_json_str(&self) -> String {
        return serde_json::to_string(&self).unwrap();
    }
}