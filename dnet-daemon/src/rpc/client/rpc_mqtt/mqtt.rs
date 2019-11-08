#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::mpsc;
use std::str::FromStr;
use std::net::IpAddr;

use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions, SecurityOptions, Notification, Publish};
use serde_json::Value;
use dnet_types::team::Team;

use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::settings::get_settings;
use crate::settings::default_settings::TINC_INTERFACE;
use crate::info::get_info;
use super::{Error, Result};
use std::time::Duration;


pub struct Mqtt {
    mqtt_options:       MqttOptions,
    daemon_event_tx:    mpsc::Sender<DaemonEvent>,
}

impl Mqtt {
    pub fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self {
        let conductor_url = &get_settings().common.conductor_url;
        let broker = conductor_url.replace("https://api", "mqtt");
        let port = 3883;

        let reconnection_options = ReconnectOptions::Always(10);

        let mut username;
        let mut password;
        loop {
            username = get_settings().common.username.clone();
            password = get_settings().common.password.clone();
            if !username.is_empty() {
                break
            }
            else {
                std::thread::sleep(Duration::from_secs(1));
            }
        }

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

    // TODO Error handle.
    pub fn run(self) -> Result<()> {
        let (mut mqtt_client, notifications) =
            MqttClient::start(self.mqtt_options.clone())
                .map_err(|_|Error::mqtt_connect_error)?;
        let username = get_settings().common.username.clone();
        mqtt_client.subscribe(&("vlan/".to_owned() + &username), QoS::AtLeastOnce)
            .map_err(|_|Error::mqtt_client_error)?;
//        mqtt_client.shutdown().unwrap();

        loop {
            if let Ok(notification) = notifications.recv() {
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
        }
    }

    fn handle_publish(&self, publish: &Publish) -> Result<()> {
        let res = String::from_utf8(publish.payload.to_vec().to_owned())

            .map_err(|_|Error::mqtt_msg_parse_failed("Request payload not utf8.".to_owned()))?
            .to_owned();
        let json: serde_json::Value = serde_json::from_str(&res).unwrap();
        info!("mqtt recv payload: {:?}", json);
        let opt_type = json.get("type")
            .ok_or(Error::mqtt_msg_parse_failed("Request type not found.".to_owned()))?
            .as_str()
            .ok_or(Error::mqtt_msg_parse_failed("Request type not string.".to_owned()))?;

        let data = json.get("data")
            .ok_or(Error::mqtt_msg_parse_failed("Request data not found.".to_owned()))?;

        match opt_type {
            "DEVICETINC_STATUS" => self.handle_host_change(&data)?,
            "ROUTER_START" => self.handle_client_start_or_stop(&data, true)?,
            "ROUTER_STOP" => self.handle_client_start_or_stop(&data, false)?,
            _ => (),
        }
        Ok(())
    }

    fn handle_host_change(&self, data: &Value) -> Result<()> {
        let ip = data.get("devcieip")
            .ok_or(Error::mqtt_msg_parse_failed("Request data.devcieip not found.".to_owned()))?
            .as_str()
            .ok_or(Error::mqtt_msg_parse_failed("Request data.devcieip wrong format.".to_owned()))
            .and_then(|ip_str| IpAddr::from_str(ip_str)
                .map_err(|_|Error::mqtt_msg_parse_failed("Request data.devcieip wrong format.".to_owned())))?;

        let info = get_info().lock().unwrap();
        let local_vip = info.tinc_info.vip;
        std::mem::drop(info);

        // mqtt local info. Skip it.
        if Some(ip) == local_vip {
            return Ok(());
        }

        let is_online = data.get("onlineStatus")
            .ok_or(Error::mqtt_msg_parse_failed("Request data.onlineStatus not found.".to_owned()))?
            .as_u64()
            .ok_or(Error::mqtt_msg_parse_failed("Request data.onlineStatus not found.".to_owned()))?;

        if is_online == 1 {
            if !sandbox::route::is_in_routing_table(&ip, 32, TINC_INTERFACE) {
                sandbox::route::add_route(&ip, 32, TINC_INTERFACE);
            }
        }
        else {
            sandbox::route::del_route(&ip, 32, TINC_INTERFACE);
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
        let info = get_info().lock().unwrap();
        let mut is_running_team = false;
        for running_team in &info.client_info.running_teams {
            if &running_team.team_id == team_id {
                is_running_team = true;
            }
        }
        if !is_running_team {
            let new_team = Team::new(team_id, team_name, vec![]);
        }

        Ok(())
    }
}