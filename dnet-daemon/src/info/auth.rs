use crate::info::get_info;

const BACKGROUND_STATUSNOTIFY: &str = "/vppn/api/v2/client/statusNotify";

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    url:            String,
    Apikey:         String,
    cookie:         String,
    Authorization:  String,
}
impl AuthInfo {
    #[allow(non_snake_case)]
    pub fn load(server_url: &str) -> Self {
        let info = get_info().lock().unwrap();
        let url = server_url.to_string() + BACKGROUND_STATUSNOTIFY;
        let Apikey = &info.proxy_info.uid;
        let cookie = &info.proxy_info.cookie;
        let Authorization = "test".to_string();
        AuthInfo {
            url,
            Apikey: Apikey.to_owned(),
            cookie: cookie.to_owned().replace("\r\n", ""),
            Authorization,
        }
    }
    pub fn to_json_str(&self) -> String {
        return serde_json::to_string(&self).unwrap();
    }
}