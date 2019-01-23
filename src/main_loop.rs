use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::sleep_ms;

use http_server_client::client::upload_proxy_status;
use net_tool::url_get;
use domain::Info;

pub fn main_loop(tinc_lock: Arc<Mutex<bool>>, conductor_url: &str, info: &Info) {
    let heartbeat_frequency = Duration::from_secs(20);
    let landmark_frequency = Duration::from_secs(15600);
    let check_tinc_frequency = Duration::from_secs(3);

    let mut now = Instant::now();
    let mut heartbeat_time = now.clone();
    let mut landmark_time = now.clone();
    let mut check_tinc_time = now.clone();

    loop {
        if now.duration_since(heartbeat_time) > heartbeat_frequency {
            upload_proxy_status(conductor_url, info);
            heartbeat_time = now.clone();
        }

        if now.duration_since(landmark_time) > landmark_frequency {
            upload_proxy_status(conductor_url, info);
            landmark_time = now.clone();
        }

        if now.duration_since(check_tinc_time) > check_tinc_frequency {
            tinc_lock.try_lock();
            upload_proxy_status(conductor_url, info);
            check_tinc_time = now.clone();
        }
        sleep_ms(1);

    }
}