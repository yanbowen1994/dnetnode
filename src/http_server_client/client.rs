//! upload proxy status

use net_tool::url_post;

pub fn upload_proxy_status(conductor_url: &str) -> bool {
    let data ;
    let res = url_post(conductor_url, data);
}
