use std::process::Command;
use sysinfo::{System, SystemExt, ProcessExt};
#[cfg(Unix)]
use sysinfo::Signal;
#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
use crate::tinc_tcp_stream::TincStream;
use super::{Error, Result, TincOperator, PID_FILENAME, TINC_BIN_FILENAME};

impl TincOperator {
    pub fn start_tinc(&self) -> Result<()> {
        if self.tinc_settings.external_boot {
            Ok(())
        }
        else {
            if let Err(Error::StopTincError) = self.hard_stop() {
                return Err(Error::StopTincError);
            }
            self.start_tinc_inner()
        }
    }

    /// 启动tinc 返回duct::handle
    fn start_tinc_inner(&self) -> Result<()> {
        let conf_tinc_home = "--config=".to_string()
            + &self.tinc_settings.tinc_home;
        let conf_pidfile = "--pidfile=".to_string()
            + &self.tinc_settings.tinc_home + PID_FILENAME;

        let args = vec![
            &conf_tinc_home[..],
            &conf_pidfile[..],
        ];

        let tincd_path;

        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
            {
                tincd_path = self.tinc_settings.tinc_home.to_string() + TINC_BIN_FILENAME;
            }
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            {
                tincd_path = TINC_BIN_FILENAME.to_string();
            }

        let duct_handle = duct::cmd(
            tincd_path,
            args
        ).unchecked();

        let _ = duct_handle.stderr_null().stdout_null().start()
            .map_err(|e| {
                log::error!("StartTincError {:?}", e.to_string());
                Error::StartTincError
            })?
            .wait()
            .map_err(|e| {
                log::error!("StartTincError {:?}", e.to_string());
                Error::StartTincError
            })?;
        Ok(())
    }

    pub fn stop_tinc(&self) -> Result<()> {
        if self.tinc_settings.external_boot {
            Ok(())
        }
        else {
            #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                {
                    let tinc_pid = self.tinc_settings.tinc_home.to_string() + PID_FILENAME;
                    if let Ok(mut tinc_stream) = TincStream::new(&tinc_pid) {
                        if let Ok(_) = tinc_stream.stop() {
                            std::mem::drop(tinc_stream);
                            // TODO async send and ipc check.
                        }
                    }
                }
            #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
                {
                    let child = std::process::Command::new("killall").arg("tincd").spawn();
                    let _ = child.and_then(|mut child| {
                        let _ = child.wait();
                        Ok(())
                    });
                }
            self.hard_stop()
        }
    }

    pub fn hard_stop(&self) -> Result<()> {
        let sys = System::new();
        for (_, info) in sys.get_process_list() {
            if info.name() == TINC_BIN_FILENAME {
                #[cfg(unix)]
                    {
                        let config_buf = "--config=".to_string()
                            + &self.tinc_settings.tinc_home.to_string();
                        if info.cmd().contains(&config_buf) {
                            if let Ok(mut res) = Command::new("kill")
                                .args(vec!["-15", &format!("{}", info.pid())])
                                .spawn() {
                                let _ = res.wait();
                            }
                        }
                    }
                #[cfg(windows)]
                    {
                        if let Ok(mut child) = Command::new("TASKKILL")
                            .args(vec!["/f", "/pid", &format!("{}", info.pid())])
                            .spawn() {
                            let _ = child.wait();
                        }

                        if let Ok(mut child) = Command::new("sc")
                            .args(vec!["delete", "tinc"])
                            .spawn() {
                            let _ = child.wait();
                        }
                    }
            }
        }
        Ok(())
    }

    pub fn restart_tinc(&mut self) -> Result<()> {
        if self.tinc_settings.external_boot {
            Ok(())
        }
        else {
            #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                {
                    match self.check_tinc_status() {
                        Ok(_) => {
                            self.stop_tinc()?;
                            self.start_tinc_inner()?;
                        },
                        Err(Error::AnotherTincRunning) => {
                            self.stop_tinc()?;
                            self.start_tinc_inner()?;
                        },
                        Err(Error::TincNeverStart) => (),
                        Err(_) => self.start_tinc_inner()?,
                    }
                    Ok(())
                }
            #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
                {
                    self.stop_tinc()?;
                    self.start_tinc()
                }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{TincOperator, TincSettings, TincRunMode};
    use crate::operator::TINC_BIN_FILENAME;
    use sysinfo::{System, SystemExt, ProcessExt, Signal};

    #[cfg(unix)]
    #[test]
    fn test_start() {
        let mut tinc_settings = TincSettings::default();
        tinc_settings.mode = TincRunMode::Client;
        TincOperator::new(tinc_settings);
        let tinc = TincOperator::instance();
        let _ = tinc.start_tinc()
            .map_err(|e|println!("{:?}", e));

        let sys = System::new();
        let mut find_tinc = false;
        for (_, info) in sys.get_process_list() {
            if info.name() == TINC_BIN_FILENAME {
                find_tinc = true;
            }
        }
        assert!(find_tinc);
        std::thread::sleep(std::time::Duration::from_secs(40));
    }

    #[cfg(unix)]
    #[test]
    fn test_start_stop() {
        let mut tinc_settings = TincSettings::default();
        tinc_settings.mode = TincRunMode::Client;
        TincOperator::new(tinc_settings);
        let tinc = TincOperator::instance();
        let _ = tinc.start_tinc()
            .map_err(|e|println!("{:?}", e));

        let sys = System::new();
        let mut find_tinc = false;
        for (_, info) in sys.get_process_list() {
            if info.name() == TINC_BIN_FILENAME {
                find_tinc = true;
            }
        }
        assert!(find_tinc);

        let mut tinc_settings = TincSettings::default();
        tinc_settings.mode = TincRunMode::Client;
        let sys = System::new();
        for (_, info) in sys.get_process_list() {
            if info.name() == TINC_BIN_FILENAME {
                #[cfg(unix)]
                    {
                        let config_buf = "--config=".to_string()
                            + &tinc_settings.tinc_home.to_string();
                        if info.cmd().contains(&config_buf) {
                            info.kill(Signal::Term);
                        }
                    }
                #[cfg(windows)]
                    info.kill(Signal::Term);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
        let sys = System::new();
        for (_, info) in sys.get_process_list() {
            if info.name() == TINC_BIN_FILENAME {
                assert!(false)
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_stop_tincd_windows() {
        let sys = System::new();
        for (_, info) in sys.get_process_list() {
            if info.name() == TINC_BIN_FILENAME {
                println!("ok");
                if let Ok(mut child) = Command::new("TASKKILL")
                    .args(vec!["/f", "/pid", "16296"])
                    .spawn() {
                    child.wait();
                }
            }
        }
    }
}