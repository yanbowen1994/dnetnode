use std::net::IpAddr;
use std::str::FromStr;

use crate::info::get_mut_info;
use crate::settings::get_settings;
use dnet_types::proxy::ProxyInfo;
use dnet_types::settings::RunMode;

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaProxy {
    authCookie:                 Option<String>,
    authId:                     Option<String>,
    authType:                   Option<String>,
    city:                       Option<String>,
    companyId:                  Option<String>,
    connections:                u32,
    country:                    Option<String>,
    edges:                      u32,
    id:                         Option<String>,
    ip:                         Option<String>,
    latitude:                   Option<String>,
    longitude:                  Option<String>,
    nodes:                      u32,
    os:                         Option<String>,
    pubkey:                     String,
    publicFlag:                 bool,
    region:                     Option<String>,
    serverPort:                 u16,
    serverType:                 Option<String>,
    sshPort:                    Option<String>,
    status:                     i32,
    tincPort:                   u16,
    userId:                     Option<String>,
    vip:                        Option<String>,
}

impl JavaProxy {
    pub fn new() -> Self {
        let mut info = get_mut_info().lock().unwrap();
        if let Err(e) = info.tinc_info.flush_connections() {
            warn!("{:?}", e);
        }

        let auth_id = info.proxy_info.auth_id.clone();
        let connections = info.tinc_info.connections.clone();
        let edges = info.tinc_info.edges.clone();
        let nodes = info.tinc_info.nodes.clone();
        let pubkey = info.tinc_info.pub_key.clone();

        let settings = get_settings();
        let ip = settings.proxy.local_ip.clone().map(|ip|ip.to_string());
        let local_port = settings.proxy.local_port;
        let tinc_port = settings.tinc.port;
        let public = settings.proxy.public;
        let server_type = match get_settings().common.mode {
            RunMode::Center => "center".to_owned(),
            RunMode::Proxy => "proxy".to_owned(),
            _ => "proxy".to_owned(),
        };
        Self {
            authCookie:     None,
            authId:         auth_id,
            authType:       None,
            city:           None,
            companyId:      None,
            connections,
            country:        None,
            edges,
            id:             None,
            ip,
            latitude:       None,
            longitude:      None,
            nodes,
            os:             None,
            pubkey,
            publicFlag:     public,
            region:         None,
            serverPort:     local_port,
            serverType:     Some(server_type),
            sshPort:        None,
            status:         0,
            tincPort:       tinc_port,
            userId:         None,
            vip:            None,
        }
    }

    pub fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }

    pub fn parse_to_proxy_info(self) -> Option<ProxyInfo> {
        let ip = self.ip
            .and_then(|ip|
                IpAddr::from_str(&ip).ok()
            )?;

        let vip = self.vip
            .and_then(|vip|
                IpAddr::from_str(&vip).ok()
            )?;

        Some(ProxyInfo{
             auth_id:          self.authId,
             auth_type:        self.authType,
             city:             self.city,
             company_id:       self.companyId,
             connections:      self.connections,
             country:          self.country,
             edges:            self.edges,
             id:               self.id,
             ip:               Some(ip),
             latitude:         self.latitude,
             longitude:        self.longitude,
             nodes:            self.nodes,
             os:               self.os,
             pubkey:           self.pubkey,
             public_flag:      self.publicFlag,
             region:           self.region,
             server_port:      self.serverPort,
             server_type:      self.serverType,
             ssh_port:         self.sshPort,
             status:           self.status,
             tinc_port:        self.tincPort,
             user_id:          self.userId,
             vip,
        })
    }
}