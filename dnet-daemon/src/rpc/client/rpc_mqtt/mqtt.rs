use std::sync::mpsc;
use std::str::FromStr;
use std::net::IpAddr;
use std::{thread, time::Duration};

use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions, SecurityOptions, Notification, Publish};

use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::settings::get_settings;

use super::{Error, Result};
use serde_json::Value;
use crate::settings::default_settings::TINC_INTERFACE;
use crate::info::get_mut_info;
use dnet_types::team::Team;

pub struct Mqtt {
    mqtt_options:       MqttOptions,
    daemon_event_tx:    mpsc::Sender<DaemonEvent>,
}

impl Mqtt {
    pub fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self {
        let conductor_url = &get_settings().common.conductor_url;
        let broker = conductor_url.replace("https://api", "mqtt");
        let port = 1883;

        let reconnection_options = ReconnectOptions::Always(10);

        let username = get_settings().common.username.clone();
        let password = get_settings().common.password.clone();

        let mqtt_user_opt = SecurityOptions::UsernamePassword(username.clone(), password);

        let mqtt_options = MqttOptions::new(&username, broker, port)
            .set_security_opts(mqtt_user_opt)
            .set_keep_alive(10)
            .set_request_channel_capacity(3)
            .set_reconnect_opts(reconnection_options)
            .set_clean_session(false);
        Self {
            mqtt_options,
            daemon_event_tx,
        }
    }

    fn run(self) -> Result<()> {
        let (mut mqtt_client, notifications) =
            MqttClient::start(self.mqtt_options.clone())
                .map_err(|_|Error::mqtt_connect_error)?;
        let username = get_settings().common.username.clone();
        mqtt_client.subscribe(&("vlan/".to_owned() + &username), QoS::AtLeastOnce)
            .map_err(|_|Error::mqtt_client_error)?;

//        thread::spawn(move || {
//            for i in 0..100 {
//                let payload = format!("publish {}", i);
//                thread::sleep(Duration::from_millis(1000));
//                mqtt_client.publish("rust_test", QoS::AtLeastOnce, false, payload)?
//            }
//        });

        let daemon_event_tx = self.daemon_event_tx.clone();

        for notification in notifications {
            match notification {
                Notification::Publish(publish) => {
                    let _ = self.handle_publish(&publish)
                        .map_err(|e|
                            error!("error: {:?}, publish: {:?}", e, publish)
                        );
                },
                Notification::PubAck(_) => (),
                Notification::PubRec(_) => (),
                Notification::PubRel(_) => (),
                Notification::PubComp(_) => (),
                Notification::SubAck(_) => (),
                _ => (),
            }
        }
        Ok(())
    }

    fn handle_publish(&self, publish: &Publish) -> Result<()> {
        let res = String::from_utf8(publish.payload.to_ascii_lowercase())
            .map_err(|_|Error::mqtt_msg_parse_failed("Request payload not utf8.".to_owned()))?;
        let json: serde_json::Value = serde_json::from_str(&res).unwrap();
        let opt_type = json.get("type")
            .ok_or(Error::mqtt_msg_parse_failed("Request type not found.".to_owned()))?
            .as_str()
            .ok_or(Error::mqtt_msg_parse_failed("Request type not string.".to_owned()))?;

        let data = json.get("data")
            .ok_or(Error::mqtt_msg_parse_failed("Request data not found.".to_owned()))?;

        match opt_type {
            "DEVICE_ONLINE" => self.handle_host_change(&data, true)?,
            "DEVICE_OFFLINE" => self.handle_host_change(&data, false)?,
            "ROUTER_START" => self.handle_client_start_or_stop(&data, true)?,
            "ROUTER_STOP" => self.handle_client_start_or_stop(&data, false)?,
            _ => (),
        }
        Ok(())
    }

    fn handle_host_change(&self, data: &Value, is_online: bool) -> Result<()> {
        let ip = data.get("ip")
            .ok_or(Error::mqtt_msg_parse_failed("Request data.ip not found.".to_owned()))?
            .as_str()
            .ok_or(Error::mqtt_msg_parse_failed("Request data.ip wrong format.".to_owned()))
            .and_then(|ip_str| IpAddr::from_str(ip_str)
                .map_err(|_|Error::mqtt_msg_parse_failed("Request data.ip wrong format.".to_owned())))?;

        if is_online {
            net_tool::route::add_route(&ip, "32", TINC_INTERFACE);
        }
        else {
            net_tool::route::del_route(&ip, "32", TINC_INTERFACE);
        }
        Ok(())
    }

    fn handle_client_start_or_stop(&self, data: &Value, is_start: bool) -> Result<()> {
        let team_id = data.get("teamId")
            .ok_or(Error::mqtt_msg_parse_failed("Request data.teamId not found.".to_owned()))?
            .as_str()
            .ok_or(Error::mqtt_msg_parse_failed("Request data.teamId wrong format.".to_owned()))?;

        let team_name = data.get("teamName")
            .ok_or(Error::mqtt_msg_parse_failed("Request data.teamName not found.".to_owned()))?
            .as_str()
            .ok_or(Error::mqtt_msg_parse_failed("Request data.teamName wrong format.".to_owned()))?;

        let _ = self.daemon_event_tx.send(DaemonEvent::DaemonInnerCmd(TunnelCommand::Connect));
        let mut info = get_mut_info().lock().unwrap();
        let mut is_running_team = false;
        for running_team in &info.client_info.running_teams {
            if &running_team.team_id == team_id {
                is_running_team = true;
            }
        }
        if !is_running_team {
            let new_team = Team::new(vec![], team_id, team_name);
        }

        Ok(())
    }
}