#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub enable:             bool,
    pub members:            Vec<TeamMember>,
    pub site_count:         u32,
    pub team_des:           String,
    pub team_id:            String,
    pub team_name:          String,
    pub terminal_count:     u32,
    pub user_count:         u32,
    pub user_id:            String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamMember {
    pub appversion:         String,
    pub city:               String,
    pub connection_limit:   String,
    pub country:            String,
    pub ip:                 String,
    pub label_name:         String,
    pub lan:                String,
    pub latitude:           String,
    pub longitude:          String,
    pub mac:                String,
    pub member_public_key:  String,
    pub member_type:        u32,
    pub proxylist:          Vec<DeviceProxy>,
    pub pubkey:             String,
    pub region:             String,
    pub serviceaddress:     String,
    pub status:             u32,
    pub team_id:            String,
    pub user_id:            String,
    pub user_name:          String,
    pub wan:                String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceProxy {
    pub city:               String,
    pub country:            String,
    pub proxy_ip:           String,
    pub proxygw:            String,
    pub proxypubkey:        String,
}
