#[derive(Debug, Clone)]
pub struct TincInfo {
    uid:String,
    proxy_ip:IpAddr,
    country:String,
    city:String,
    region:String,
    os:String,
    last_heartbeat:String,
    pub_key:String,
}