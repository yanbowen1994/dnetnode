use std::process::{Command, Stdio};

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