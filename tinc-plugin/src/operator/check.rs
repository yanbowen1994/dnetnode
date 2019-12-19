use std::net::IpAddr;

use crate::{TincStream, TincTools};

use super::{Error, Result, TincOperator};
use super::PID_FILENAME;

impl TincOperator {
    pub fn check_tinc_status(&self) -> Result<()> {
        TincTools::get_tinc_pid_by_sys(
            #[cfg(unix)]
            &self.tinc_settings.tinc_home
        )
            .ok_or(Error::TincNotExist)
            .and_then(|_|Ok(()))
    }

    pub fn check_tinc_listen(&self) -> Result<()> {
        let pid_file = self.tinc_settings.tinc_home.clone() + PID_FILENAME;
        TincStream::new(&pid_file)
            .map_err(|_|Error::TincNotExist)?
            .connect_test()
            .map_err(|_|Error::TincNotExist)
    }

    pub fn get_tinc_connect_nodes(&self) -> Result<Vec<IpAddr>> {
        let pid_file = std::path::Path::new(&self.tinc_settings.tinc_home)
            .join(PID_FILENAME);
        if !pid_file.as_path().is_file() {
            return Err(Error::PidfileNotExist);
        }
        let pid_file = pid_file
            .to_str()
            .unwrap()
            .to_string();
        let mut tinc_stream = TincStream::new(&pid_file)
            .map_err(|_|Error::PidfileNotExist)?;
        let source_connections = tinc_stream.dump_edges()
            .map_err(|_|Error::TincNotExist)?;
        let nodes = source_connections.into_iter()
            .filter_map(|source| {
                if !source.from.contains("proxy") {
                    if source.from.len() > 3 {
                        TincTools::get_vip_by_filename(&source.from)
                    }
                    else {
                        None
                    }
                }
                else {
                    None
                }
            })
            .collect::<Vec<IpAddr>>();
        Ok(nodes)
    }
}