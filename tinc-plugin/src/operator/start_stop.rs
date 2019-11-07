use crate::TincStream;
use super::{Error, Result, TincOperator, PID_FILENAME, TINC_BIN_FILENAME};

impl TincOperator {
    pub fn start_tinc(&mut self) -> Result<()> {
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
        #[cfg(any(target_arch = "arm", feature = "router_debug"))]
            {
                let _ = self.stop_tinc();
            }

        self.start_tinc_inner()
    }

    /// 启动tinc 返回duct::handle
    fn start_tinc_inner(&mut self) -> Result<()> {
        let mut mutex_tinc_handle = self.tinc_handle.lock().unwrap();

        let conf_tinc_home = "--config=".to_string()
            + &self.tinc_settings.tinc_home;
        let conf_pidfile = "--pidfile=".to_string()
            + &self.tinc_settings.tinc_home + PID_FILENAME;

        let mut args = vec![
            &conf_tinc_home[..],
            &conf_pidfile[..],
            "--no-detach",
        ];

        let tinc_debug_level = format!("{}", self.tinc_settings.tinc_debug_level);
        let log_file = self.tinc_settings.tinc_home.clone() + "tinc.log";
        if self.tinc_settings.tinc_debug_level != 0 {
            args.push("-d");
            args.push(&tinc_debug_level);
            args.push("--logfile");
            args.push(&log_file);
        }

        let duct_handle: duct::Expression = duct::cmd(
            &(self.tinc_settings.tinc_home.to_string() + TINC_BIN_FILENAME),
            args)
            .unchecked();

        let tinc_handle = duct_handle.stderr_capture().stdout_null().start()
            .map_err(|e| {
                log::error!("StartTincError {:?}", e.to_string());
                Error::StartTincError
            })?;

        *mutex_tinc_handle = Some(tinc_handle);
        Ok(())
    }

    pub fn stop_tinc(&mut self) -> Result<()> {
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
                self.tinc_handle
                    .lock()
                    .unwrap()
                    .as_ref()
                    .ok_or(Error::TincNeverStart)
                    .and_then(|child| {
                        child.kill().map_err(|_| Error::StopTincError)?;
                        // clean out put.
                        for _i in 0..10 {
                            let _ = child.try_wait();
                        }
                        Ok(())
                    })?;

                let handle = self.tinc_handle.lock().unwrap().take();
                std::mem::drop(handle);
                Ok(())
            }
        #[cfg(any(target_arch = "arm", feature = "router_debug"))]
            {
                let child = std::process::Command::new("killall").arg("tincd").spawn();
                let _ = child.and_then(|mut child| {
                    let _ = child.wait();
                    Ok(())
                });
                Ok(())
            }
    }

    pub fn restart_tinc(&mut self)
                        -> Result<()> {
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
        #[cfg(any(target_arch = "arm", feature = "router_debug"))]
            {
                self.stop_tinc()?;
                self.start_tinc()
            }
    }

    // if outside kill tinc clean tinc_handle
    pub fn clean_tinc_output(&self) {
        let handle = self.tinc_handle.lock().unwrap();
        if handle.is_some() {
            let _ = handle.as_ref().unwrap().wait();
        }
    }
}