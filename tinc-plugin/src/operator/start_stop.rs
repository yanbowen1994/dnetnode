use crate::TincStream;
#[cfg(unix)]
use crate::TincTools;
use super::{Error, Result, TincOperator, PID_FILENAME, TINC_BIN_FILENAME};
use std::process::Command;

impl TincOperator {
    pub fn start_tinc(&self) -> Result<()> {
        if self.tinc_settings.external_boot {
            Ok(())
        }
        else {
            #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                {
                    match self.check_tinc_listen() {
                        Ok(_) => {
                            if let Err(Error::StopTincError) = self.stop_tinc() {
                                return Err(Error::StopTincError);
                            }
                        },
                        Err(_) => (),
                    }
                }
            #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
                {
                    let _ = self.stop_tinc();
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

        let _ = Command::new(&(self.tinc_settings.tinc_home.to_string() + TINC_BIN_FILENAME))
            .args(args)
            .spawn()
            .map_err(|e| {
                log::error!("StartTincError {:?}", e.to_string());
                println!("StartTincError {:?}", e.to_string());
                Error::StartTincError
            })?
            .wait();
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
                            // TODO async send and ipc check.
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            ()
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
            // TODO windows
            #[cfg(unix)]
                {
                    if let Some(pid) = TincTools::get_tinc_pid_by_sys(&self.tinc_settings.tinc_home) {
                        {
                            if let Ok(mut res) = Command::new("kill")
                                .args(vec!["-15", &format!("{}", pid)])
                                .spawn() {
                                let _ = res.wait();
                            }
                        }
                    }
                    #[cfg(windows)]
                    {

                    }
                }
            Ok(())
        }
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

    #[test]
    fn test_start() {
        let mut tinc_settins = TincSettings::default();
        tinc_settins.mode = TincRunMode::Center;
        TincOperator::new(tinc_settins);
        let tinc = TincOperator::instance();
        tinc.start_tinc()
            .map_err(|e|println!("{:?}", e));
    }
}