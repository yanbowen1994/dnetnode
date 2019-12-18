use std::process::{Command, Stdio};
use std::net::IpAddr;

use crate::{TincStream, TincTools};

use super::{Error, Result, TincOperator};
use super::{PID_FILENAME, TINC_MEMORY_LIMIT, TINC_ALLOWED_OUT_MEMORY_TIMES};

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
        let source_connections = tinc_stream.dump_nodes()
            .map_err(|_|Error::TincNotExist)?;
        let connections = source_connections.into_iter()
            .filter_map(|source| {
                if !source.node.contains("proxy") {
                    if source.via.len() > 3 {
                        TincTools::get_vip_by_filename(&source.node)
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
        Ok(connections)
    }

    pub fn check_tinc_memory(&mut self) -> Result<()> {
        let mut res1 = Command::new("ps")
            .arg("-aux")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let res2 = Command::new("grep")
            .arg("tincd")
            .stdin(res1.stdout.take().unwrap())
            .output()
            .unwrap();
        let res = String::from_utf8_lossy( &res2.stdout).to_string();
        let res_vec = res.split_ascii_whitespace().collect::<Vec<&str>>();

        if res_vec.len() < 4 {
            return Err(Error::TincNotExist);
        }

        let memory: f32 = res_vec[3].parse().map_err(|_|Error::TincNotExist)?;

        if memory > TINC_MEMORY_LIMIT {
            if self.tinc_out_memory_times >= TINC_ALLOWED_OUT_MEMORY_TIMES {
                self.tinc_out_memory_times = 0;
                return Err(Error::OutOfMemory);
            }
            else {
                self.tinc_out_memory_times += 1;
            }
        }
        Ok(())
    }
}