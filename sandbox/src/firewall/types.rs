use std::net::IpAddr;

pub struct IptablesRule {
    pub in_:            String,
    pub out:            String,
    pub src:            String,
    pub dst:            String,
    pub target:         String,
}