use std::net::IpAddr;
use crate::route::get_default_route;
use dnet_types::team::NetSegment;

pub fn get_default_interface() -> Option<NetSegment> {
    let default_route = get_default_route();


    #[cfg(unix)]
        {
            let cmd = duct::cmd!(
                "route", "-n", "get", "default");
            let res = cmd.read().map_err(|e| Error::GetDefaultGatewayError(e.to_string()))?;
            let res: Vec<&str> = res.split("gateway: ").collect();
            if res.len() < 1 {
                return Err(Error::GetDefaultGatewayError("route -n get default not find gateway:".to_string()));
            }
            let res: Vec<&str> = res[1].split("\n").collect();
            let res = res[0];
            IpAddr::from_str(res).map_err(|e| Error::GetDefaultGatewayError(e.to_string()))
        }
    #[cfg(target_os = "windows")]
        {
            let cmd = ::std::process::Command::new("route")
                .args(vec![
                    "print",
                ])
                .output()
                .expect("sh command failed to start");

            let stdout = cmd.stdout;

            let mut res = vec![];
            for i in stdout {
                if i < 32 || 126 < i {
                    continue
                }
                else {
                    res.push(i);
                }
            }

            let mut res = String::from_utf8(res)
                .map_err(|e|Error::GetDefaultGatewayError(e.to_string()))?;
            for _ in 0..5 {
                res = res.replace("    ", " ").replace("  ", " ");
            }

            let res: Vec<&str> = res.split("0.0.0.0").collect();
            if res.len() < 2 {
                return Err(Error::GetDefaultGatewayError("route print not find 0.0.0.0 route.".to_string()));
            }

            let res: Vec<&str> = res[2].split(" ").collect();
            let default_gateway_str = res[2];
            let default_gateway = IpAddr::from_str(default_gateway_str)
                .map_err(|e| Error::GetDefaultGatewayError(e.to_string()
                    + "\n" + "route print not find 0.0.0.0 route"))?;
            return Ok(default_gateway);
        }
}