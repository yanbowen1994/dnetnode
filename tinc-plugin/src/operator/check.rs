use std::process::{Command, Stdio};

use crate::TincStream;

use super::{Error, Result, TincOperator};
use super::{PID_FILENAME, TINC_MEMORY_LIMIT, TINC_ALLOWED_OUT_MEMORY_TIMES};

impl TincOperator {
    #[cfg(unix)]
    pub fn check_tinc_status(&self) -> Result<()> {
        let mut res1;
        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
            {
                res1 = Command::new("ps")
                    .arg("-aux")
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();
            }
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            {
                res1 = Command::new("ps")
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();
            }

        let res2 = Command::new("grep")
            .arg("tincd")
            .stdin(res1.stdout.take().unwrap())
            .output()
            .unwrap();
        let res = String::from_utf8_lossy( &res2.stdout).to_string();
        let _ = res1.wait();
        if !res.contains("config") {
            self.clean_tinc_output();
            return Err(Error::TincNotExist);
        }

        return Ok(());
    }

    #[cfg(windows)]
    pub fn check_tinc_status(&self) -> Result<()> {
        if windows::find_process("tincd.exe").map_err(|_|Error::TincNotExist)? {
            Ok(())
        }
        else {
            Err(Error::TincNeverStart)
        }
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

#[cfg(windows)]
mod windows {
    extern crate winapi;
    use std::io::Error;

    use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, TH32CS_SNAPPROCESS, Process32First, Process32Next, PROCESSENTRY32};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::winnt::HANDLE;
    use winapi::shared::minwindef::{TRUE, MAX_PATH};
    pub fn find_process(name: &str) -> Result<bool, Error> {

        let snapshot_handle: HANDLE = unsafe {
            // https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
        };
        if snapshot_handle == INVALID_HANDLE_VALUE {
            Err(Error::last_os_error())
        } else {
            let mut process: PROCESSENTRY32 = unsafe { std::mem::zeroed() };
            // https://stackoverflow.com/questions/29346365/process32first-is-not-returning-true-even-if-process-is-running
            // > Before calling the Process32First function, set this member to sizeof(PROCESSENTRY32).
            //   If you do not initialize dwSize, Process32First fails.
            process.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

            if unsafe { Process32First(snapshot_handle, &mut process) } == TRUE {
                if compare(&process, name) {
                    return Ok(true);
                }

                while unsafe { Process32Next(snapshot_handle, &mut process) } == TRUE {
                    if compare(&process, name) {
                        return Ok(true);
                    }
                }

                unsafe {
                    CloseHandle(snapshot_handle);
                }

            } else {
                unsafe {
                    CloseHandle(snapshot_handle);
                }

                return Err(Error::last_os_error());
            }
            return Ok(false)
        }
    }

    fn compare(process: &PROCESSENTRY32, find_name: &str) -> bool {
        let name = process_name(&process);
        name == find_name
    }

    fn process_name(process: &PROCESSENTRY32) -> &str {
        let exe_file = &process.szExeFile;
        let ptr = exe_file as *const i8 as *const u8;
        let len = exe_file.iter().position(|&ch| ch == 0).unwrap_or(MAX_PATH);
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        std::str::from_utf8(slice).unwrap()
    }
}