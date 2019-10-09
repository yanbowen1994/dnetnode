use std::sync::mpsc;
use std::{thread, time::Duration};

extern crate rumqtt;
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions, SecurityOptions};
use crate::daemon::DaemonEvent;
use crate::settings::get_settings;

pub type Result<T> = std::result::Result<T, rumqtt::Error>;

pub struct Mqtt {
    mqtt_options:       MqttOptions,
    daemon_event_tx:    mpsc::Sender<DaemonEvent>,
}

impl Mqtt {
    pub fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self {
        let settings = get_settings();
        let broker = "mqtt-qa.vlan.cn";
        let port = 1883;

        let reconnection_options = ReconnectOptions::Always(10);

        let username = get_settings().common.username.clone();
        let password = get_settings().common.password.clone();

        let mqtt_user_opt = SecurityOptions::UsernamePassword(username, password);

        let mqtt_options = MqttOptions::new("test-pubsub1", broker, port)
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
        let (mut mqtt_client, notifications) = MqttClient::start(self.mqtt_options)?;
        mqtt_client.subscribe("rust_test", QoS::AtLeastOnce).unwrap();

        thread::spawn(move || {
            for i in 0..100 {
                let payload = format!("publish {}", i);
                thread::sleep(Duration::from_millis(1000));
                mqtt_client.publish("rust_test", QoS::AtLeastOnce, false, payload)?
            }
        });

        let daemon_event_tx = self.daemon_event_tx.clone();

        for notification in notifications {
//            notification.
//            daemon_event_tx
//            println!("{:?}", notification)
        }
        Ok(())
    }
}