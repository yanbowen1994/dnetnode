pub enum MqttCmdType {
    DeviceOnline,
    DeviceOffline,
    StartTunnelTeam,
    StopTunnelTeam,
}

pub struct MqttCmd {
    cmd_type: MqttCmdType,

}