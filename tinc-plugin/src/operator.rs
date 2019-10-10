#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_patterns)]

use std::ffi::OsString;
use std::fs;
use std::io::{self, Write, Read};
use std::sync::Mutex;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

use duct;
use openssl::rsa::Rsa;

use crate::{TincInfo, TincRunMode, TincStream};
use std::path::Path;
use std::time::SystemTime;

/// Results from fallible operations on the Tinc tunnel.
pub type Result<T> = std::result::Result<T, Error>;

static mut EL: *mut TincOperator = 0 as *mut _;

#[cfg(unix)]
const TINC_BIN_FILENAME: &str = "tincd";
#[cfg(windows)]
const TINC_BIN_FILENAME: &str = "tincd.exe";

const PRIV_KEY_FILENAME: &str = "rsa_key.priv";

pub const PUB_KEY_FILENAME: &str = "rsa_key.pub";

#[cfg(unix)]
const TINC_UP_FILENAME: &str = "tinc-up";
#[cfg(windows)]
const TINC_UP_FILENAME: &str = "tinc-up.bat";

#[cfg(unix)]
const TINC_DOWN_FILENAME: &str = "tinc-down";
#[cfg(windows)]
const TINC_DOWN_FILENAME: &str = "tinc-down.bat";

#[cfg(unix)]
const HOST_UP_FILENAME: &str = "host-up";
#[cfg(windows)]
const HOST_UP_FILENAME: &str = "host-up.bat";

#[cfg(unix)]
const HOST_DOWN_FILENAME: &str = "host-down";
#[cfg(windows)]
const HOST_DOWN_FILENAME: &str = "host-down.bat";

pub const PID_FILENAME: &str = "tinc.pid";

const TINC_AUTH_PATH: &str = "auth/";

const TINC_AUTH_FILENAME: &str = "auth.txt";

/// Errors that can happen when using the Tinc tunnel.
#[derive(err_derive::Error, Debug)]
#[allow(non_camel_case_types)]
pub enum Error {
    #[error(display = "Tinc Info Proxy Ip Not Found")]
    TincInfo_connect_to_is_empty,

    #[error(display = "Tinc Info Proxy Ip Not Found")]
    TincInfoProxyIpNotFound,

    #[error(display = "Get local ip before Rpc get_online_proxy")]
    GetLocalIpBeforeRpcGetOnlineProxy,

    /// Unable to start
    #[error(display = "duct can not start tinc")]
    NeverInitOperator,

    /// Unable to start
    #[error(display = "duct can not start tinc")]
    StartTincError,

    #[error(display = "duct can not start tinc")]
    AnotherTincRunning,

    /// Unable to stop
    #[error(display = "duct can not stop tinc")]
    StopTincError,

    /// tinc process not exist
    #[error(display = "tinc pidfile not exist")]
    PidfileNotExist,

    /// tinc process not exist
    #[error(display = "tinc process not exist")]
    TincNotExist,

    /// If should restart tinc, like config change, that error will skip to restart.
    #[error(display = "tinc process not start")]
    TincNeverStart,

    /// tinc host file not exist
    #[error(display = "tinc host file not exist")]
    FileNotExist(String),

    /// Failed create file
    #[error(display = "Failed create file")]
    FileCreateError(String),

    /// Tinc can't create key pair
    #[error(display = "Tinc can't create key pair")]
    CreatePubKeyError,

    /// Invalid tinc info
    #[error(display = "Invalid tinc info")]
    TincInfoError(String),

    /// Error while running "ip route".
    #[error(display = "Error while running \"ip route\"")]
    FailedToRunIpRoute(#[error(cause)] io::Error),

    /// Io error
    #[error(display = "Io error")]
    IoError(String),

    /// No wan dev
    #[error(display = "No wan dev")]
    NoWanDev,

    /// Address loaded from file is invalid
    #[error(display = "Address loaded from file is invalid")]
    ParseLocalIpError(#[error(cause)] std::net::AddrParseError),

    /// Address loaded from file is invalid
    #[error(display = "Address loaded from file is invalid")]
    ParseLocalVipError(#[error(cause)] std::net::AddrParseError),

    ///
    #[error(display = "Get default gateway error")]
    GetDefaultGatewayError(String),

    ///
    #[error(display = "Permissions error")]
    PermissionsError(#[error(cause)] std::io::Error),

    ///
    #[error(display = "Permissions error")]
    VnicNotFind(String),
}

trait Proxy {
    fn create_pub_key(&self) -> Result<()>;
    fn set_local_pub_key(&mut self, pub_key: &str) -> Result<()>;
}

/// Tinc operator
pub struct TincOperator {
    tinc_home:              String,
    mutex:                  Mutex<i32>,
    mode:                   TincRunMode,
    tinc_handle:            Mutex<Option<duct::Handle>>,
}

impl TincOperator {
    /// 获取tinc home dir 创建tinc操作。
    pub fn new(tinc_home: &str, mode: TincRunMode) {
        let operator = TincOperator {
            tinc_home:      tinc_home.to_string(),
            mutex:          Mutex::new(0),
            mode,
            tinc_handle:    Mutex::new(None),
        };

        unsafe {
            EL = Box::into_raw(Box::new(operator));
        }
    }

    pub fn mut_instance() ->  &'static mut Self {
        unsafe {
            if EL == 0 as *mut _ {
                panic!("Get tinc Operator instance, before init");
            }
            &mut *EL
        }
    }

    pub fn instance() ->  &'static Self {
        unsafe {
            if EL == 0 as *mut _ {
                panic!("Get tinc Operator instance, before init");
            }
            & *EL
        }
    }

    pub fn is_inited() -> bool {
        unsafe {
            if EL == 0 as *mut _ {
                return false;
            }
        }
        return true;
    }

    pub fn start_tinc(&mut self) -> Result<()> {
        match self.check_tinc_status() {
            Ok(_) => self.stop_tinc()?,
            Err(_) => (),
        }
        self.start_tinc_inner()
    }

    /// 启动tinc 返回duct::handle
    fn start_tinc_inner(&mut self) -> Result<()> {
        let mut mutex_tinc_handle = self.tinc_handle.lock().unwrap();

        let conf_tinc_home = "--config=".to_string()
            + &self.tinc_home;
        let conf_pidfile = "--pidfile=".to_string()
            + &self.tinc_home + PID_FILENAME;

        // Set tinc running dir, for tincd link lib openssl.
        let current_dir = std::env::current_dir()
            .map_err(|e|Error::IoError(e.to_string()))?;
        std::env::set_current_dir(self.tinc_home.to_string())
            .map_err(|e|Error::IoError(e.to_string()))?;
        let duct_handle: duct::Expression = duct::cmd(
            OsString::from(self.tinc_home.to_string() + TINC_BIN_FILENAME),
            vec![
                &conf_tinc_home[..],
                &conf_pidfile[..],
                "--no-detach",
            ])
            .unchecked();
        std::env::set_current_dir(current_dir)
            .map_err(|e|Error::IoError(e.to_string()))?;

        let mut tinc_handle = duct_handle.stderr_capture().stdout_null().start()
            .map_err(|e| {
                log::error!("StartTincError {:?}", e.to_string());
                Error::StartTincError
            })?;

        let _ = tinc_handle.try_wait();

        *mutex_tinc_handle = Some(tinc_handle);
        Ok(())
    }

    pub fn stop_tinc(&mut self) -> Result<()> {
        let tinc_pid = self.tinc_home.to_string() + PID_FILENAME;
        if let Ok(mut tinc_stream) = TincStream::new(&tinc_pid) {
            if let Ok(_) = tinc_stream.stop() {
                return Ok(());
            }
        }
        self.tinc_handle
            .lock()
            .unwrap()
            .as_ref()
            .ok_or(Error::StopTincError)
            .and_then(|child|
                child.kill().map_err(|_| Error::StopTincError)
            )?;

        *self.tinc_handle.lock().unwrap() = None;
        Ok(())
    }

    pub fn check_tinc_status(&self) -> Result<()> {
        let mut tinc_handle = self.tinc_handle
            .lock()
            .unwrap()
            .as_ref()
            .ok_or(Error::TincNeverStart)
            .and_then(|child| {
                let out = child.try_wait()
                    .map_err(|_|Error::TincNotExist)?;

                if let None = out {
                    return Ok(())
                }

                if let Some(mut out) = out {
                    if let Some(code) = out.status.code() {
                        let mut error = String::from_utf8_lossy(&out.stderr).to_string();
                        if error.contains("Address already in use") {
                            error = "port 50069 already in use".to_owned();
                        }
                        error!("code:{} error:{:?}", code, error);
                        return Err(Error::TincNotExist);
                    }
                }
                return Ok(());
            })?;

        return Ok(());
    }

    pub fn restart_tinc(&mut self)
        -> Result<()> {
        match self.check_tinc_status() {
            Ok(_) => self.stop_tinc()?,
            Err(Error::TincNeverStart) => (),
            Err(_) => self.start_tinc_inner()?,
        }
        Ok(())
    }

    /// 根据IP地址获取文件名
    pub fn get_filename_by_ip(is_proxy: bool, ip: &str) -> String {
        let splits = ip.split(".").collect::<Vec<&str>>();
        let mut filename = String::new();
        if is_proxy {
            if splits.len() >= 4 {
                filename = "proxy".to_string() + "_";
                filename.push_str(splits[0]);
                filename.push_str("_");
                filename.push_str(splits[1]);
                filename.push_str("_");
                filename.push_str(splits[2]);
                filename.push_str("_");
                filename.push_str(splits[3]);
            }
            else {
                filename.push_str("vpnserver");
            }
        }
        else if splits.len() > 3 {
            filename.push_str(splits[1]);
            filename.push_str("_");
            filename.push_str(splits[2]);
            filename.push_str("_");
            filename.push_str(splits[3]);
        }
        filename
    }

    pub fn check_pub_key(&self) -> bool {
        let pubkey_path = self.tinc_home.to_owned() + PUB_KEY_FILENAME;
        let path = Path::new(&pubkey_path);
        if let Ok(fs) = std::fs::metadata(path) {
            if let Ok(time) = fs.modified() {
                if let Ok(now) = SystemTime::now().duration_since(time) {
                    if now.as_secs() / 60 / 60 / 24 < 30 {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    /// openssl Rsa 创建2048位密钥对, 并存放到tinc配置文件中
    pub fn create_pub_key(&self) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        let mut write_priv_key_ok = false;
        if let Ok(key) = Rsa::generate(2048) {
            if let Ok(priv_key) = key.private_key_to_pem() {
                if let Ok(priv_key) = String::from_utf8(priv_key) {
                    let mut file = fs::File::create(
                        self.tinc_home.to_string() + PRIV_KEY_FILENAME)
                        .map_err(|e|
                            Error::FileCreateError((self.tinc_home.to_string() + PRIV_KEY_FILENAME)
                                + " " + &e.to_string()))?;
                    file.write_all(priv_key.as_bytes())
                        .map_err(|_|Error::CreatePubKeyError)?;
                    drop(file);

                    write_priv_key_ok = true;
                }
            }
            if let Ok(pub_key) = key.public_key_to_pem() {
                if let Ok(pub_key) = String::from_utf8(pub_key) {
                    let path = self.tinc_home.to_string() + PUB_KEY_FILENAME;
                    let mut file = fs::File::create(&path)
                        .map_err(|e|Error::FileCreateError(path.clone() + " " + &e.to_string()))?;
                    file.write_all(pub_key.as_bytes())
                        .map_err(|_|Error::CreatePubKeyError)?;
                    drop(file);
                    if write_priv_key_ok {
                        return Ok(());
                    }
                }
            }
        }
        Err(Error::CreatePubKeyError)
    }

    /// 从pub_key文件读取pub_key
    pub fn get_local_pub_key(&self) -> Result<String> {
        let _guard = self.mutex.lock().unwrap();
        let path = self.tinc_home.clone() + PUB_KEY_FILENAME;
        let mut file =  fs::File::open(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        Ok(buf)
    }

    /// 修改本地公钥
    pub fn set_local_pub_key(&self, pub_key: &str) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        let path = self.tinc_home.clone() + PUB_KEY_FILENAME;
        let mut file =  fs::File::create(path.clone())
            .map_err(|_|Error::CreatePubKeyError)?;
        file.write(pub_key.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        return Ok(());
    }

    /// 获取本地tinc虚拟ip
    pub fn get_local_vip(&self) -> Result<IpAddr> {
        let _guard = self.mutex.lock().unwrap();
        let mut out = String::new();

        let path = self.tinc_home.clone() + TINC_UP_FILENAME;
        let mut file = fs::File::open(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;

        let mut res = String::new();
        file.read_to_string(&mut res)
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            let res: Vec<&str> = res.split("vpngw=").collect();
        #[cfg(windows)]
            let res: Vec<&str> = res.split("addr=").collect();
        if res.len() > 1 {
            let res = res[1].to_string();
            #[cfg(unix)]
            let res: Vec<&str> = res.split("\n").collect();
            #[cfg(windows)]
            let res: Vec<&str> = res.split(" mask").collect();
            if res.len() > 1 {
                out = res[0].to_string();
            }
        }
        Ok(IpAddr::from(Ipv4Addr::from_str(&out).map_err(Error::ParseLocalVipError)?))
    }

    /// 通过Info修改tinc.conf
    fn set_tinc_conf_file(&self, tinc_info: &TincInfo) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();

        let (is_proxy, name_ip) = match self.mode {
            TincRunMode::Proxy => {
                let name_ip = tinc_info.ip.clone()
                    .ok_or(Error::TincInfoProxyIpNotFound)?;
                (true, name_ip)
            },
            TincRunMode::Client => (false, tinc_info.vip.clone()),
        };

        let name = Self::get_filename_by_ip(is_proxy,
                                            &name_ip.to_string());

        let mut connect_to: Vec<String> = vec![];
        for online_proxy in tinc_info.connect_to.clone() {
            let online_proxy_name = Self::get_filename_by_ip(true,
                                                             &online_proxy.ip.to_string());
            connect_to.push(online_proxy_name);
        }


        let mut buf_connect_to = String::new();
        for other in connect_to {
            let buf = "ConnectTo = ".to_string() + &other + "\n";
            buf_connect_to += &buf;
        }

        let buf;
        #[cfg(target_os = "linux")]
        {
            buf = "Name = ".to_string() + &name + "\n"
                + &buf_connect_to
                + "DeviceType=tap\n\
                   Mode=switch\n\
                   Interface=dnet\n\
                   BindToAddress = * 50069\n\
                   ProcessPriority = high\n\
                   PingTimeout=10\n\
                   Device = /dev/net/tun\n\
                   AutoConnect=no\n\
                   MaxConnectionBurst=1000\n";
        }
        #[cfg(target_os = "macos")]
        {
            buf = "Name = ".to_string() + &name + "\n"
                + &buf_connect_to
                + "DeviceType=tap\n\
                   Mode=switch\n\
                   Interface=dnet\n\
                   BindToAddress = * 50069\n\
                   ProcessPriority = high\n\
                   PingTimeout=10\n\
                   Device = /dev/tap0\n\
                   AutoConnect=no\n\
                   MaxConnectionBurst=1000\n";
        }
        #[cfg(windows)]
        {
            buf = "Name = ".to_string() + &name + "\n"
                + &buf_connect_to
                + "DeviceType=tap\n\
                   Mode=switch\n\
                   Interface=dnet\n\
                   BindToAddress = * 50069\n\
                   ProcessPriority = high\n\
                   PingTimeout=10\n\
                   AutoConnect=no\n\
                   MaxConnectionBurst=1000\n";
        }

        let path = self.tinc_home.clone() + "tinc.conf";
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::FileCreateError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        return Ok(());
    }

    /// 检查info中的配置, 并与实际运行的tinc配置对比, 如果不同修改tinc配置,
    /// 如果自己的vip修改,重启tinc
//    pub fn check_info(&mut self, tinc_info: &TincInfo) -> Result<()> {
//        let mut need_restart = false;
//        {
//            let file_vip = self.get_local_vip()?;
//            if file_vip != tinc_info.vip {
//                log::debug!("tinc operator check_info local {}, remote {}",
//                       file_vip,
//                       tinc_info.vip.to_string());
//
//                self.set_tinc_up(&tinc_info)?;
//
//                self.set_hosts(true,
//                                   &tinc_info.ip.to_string(),
//                                   &tinc_info.pub_key)?;
//
//                need_restart = true;
//            }
//        }
//        {
//            for online_proxy in tinc_info.connect_to.clone() {
//                self.set_hosts(true,
//                                   &online_proxy.ip.to_string(),
//                                   &online_proxy.pubkey)?;
//            }
//        }
//
//        self.check_self_hosts_file(&self.tinc_home, &tinc_info)?;
//        self.set_hosts(
//            true,
//            &tinc_info.ip.to_string(),
//            &tinc_info.pub_key)?;
//
//        if need_restart {
//            self.set_tinc_conf_file(&tinc_info)?;
//            self.stop_tinc()?;
//        }
//        return Ok(());
//    }

    /// 添加hosts文件
    /// if is_proxy{ 文件名=proxy_10_253_x_x }
    /// else { 文件名=虚拟ip后三位b_c_d }
    pub fn set_hosts(&self,
                     is_proxy: bool,
                     ip: &str,
                     pubkey: &str)
        -> Result<()>
    {
        let _guard = self.mutex.lock().unwrap();
        {
            let mut buf;
            if is_proxy {
                buf = "Address=".to_string() + ip + "\n"
                    + "Port=50069\n"
                    + pubkey;
            }
            else {
                buf = pubkey.to_string();
            }
            let file_name = Self::get_filename_by_ip(is_proxy, ip);

            let path = self.tinc_home.clone() + "hosts/" + &file_name;
            let mut file = fs::File::create(path.clone())
                .map_err(|e|Error::FileCreateError(path.clone() + " " + &e.to_string()))?;
            file.write(buf.as_bytes())
                .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        }
        Ok(())
    }

    /// 检测自身hosts文件,是否正确
//    pub fn check_self_hosts_file(&self, tinc_home: &str, tinc_info: &TincInfo) -> Result<()> {
//        let _guard = self.mutex.lock().unwrap();
//        let ip = tinc_info.ip.to_string();
//
//        let is_proxy = match self.mode {
//            TincRunMode::Proxy => true,
//            TincRunMode::Client => false,
//        };
//        let filename = Self::get_filename_by_ip(is_proxy, &ip);
//
//        let path = tinc_home.to_string()
//            + "hosts/"
//            + "proxy_"
//            + &filename;
//        fs::File::create(path.clone())
//            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
//        Ok(())
//    }

    /// Load local tinc config file vpnserver for tinc vip and pub_key.
    /// Success return true.
//    pub fn load_local(&mut self, tinc_home: &str) -> io::Result<TincInfo> {
//        let _guard = self.mutex.lock().unwrap();
//        let mut tinc_info = TincInfo::new();
//        {
//            let mut res = String::new();
//            let mut _file = fs::File::open(tinc_home.to_string() + PUB_KEY_FILENAME)?;
//            _file.read_to_string(&mut res).map_err(Error::IoError)?;
//            tinc_info.pub_key = res.clone();
//        }
//        {
//            tinc_info.vip = self.get_local_vip()?
//
//        }
//        return Err(Error::L);
//    }

    pub fn set_info_to_local(&mut self, info: &TincInfo) -> Result<()> {
        self.set_tinc_conf_file(info)?;
        let is_proxy = match self.mode {
            TincRunMode::Proxy => true,
            TincRunMode::Client => false,
        };

        self.set_tinc_up(&info)?;
        self.set_tinc_down(info)?;
        self.set_host_up()?;
        self.set_host_down()?;

        for online_proxy in info.connect_to.clone() {
            self.set_hosts(true,
                           &online_proxy.ip.to_string(),
                           &online_proxy.pubkey)?;
        };

        let ip;
        if is_proxy {
            ip = info.ip
                .ok_or(Error::TincInfoProxyIpNotFound)?
                .to_string();
        }
        else {
            ip = info.vip.to_string();
        }
        self.set_hosts(is_proxy, &ip, &info.pub_key)
    }

    fn set_tinc_up(&self, tinc_info: &TincInfo) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();

        let netmask = match self.mode {
            TincRunMode::Proxy => "255.0.0.0",
            TincRunMode::Client => "255.255.255.255",
        };

        let mut buf;

        #[cfg(target_os = "linux")]
        {
            buf = "#! /bin/bash\n\
            dev=dnet\n\
            vpngw=".to_string() + &tinc_info.vip.to_string() + "\n" +
            "ifconfig ${dev} ${vpngw} netmask " + netmask;

            if TincRunMode::Client == self.mode {
                if tinc_info.connect_to.is_empty() {
                    return Err(Error::TincInfo_connect_to_is_empty)
                }

                buf = buf + "\n"
                    + "route add -host " + &tinc_info.connect_to[0].ip.to_string() + " gw _gateway";
                buf = buf + "\n"
                    + "route add -host 10.255.255.254 dev dnet";
                buf = buf + "\n"
                    + "route add default gw " + &tinc_info.connect_to[0].vip.to_string();
            }

            buf = buf + "\n" + &self.tinc_home + "tinc-report -u";
        }
        #[cfg(target_os = "macos")]
        {
            buf = "#! /bin/bash\n\
                   dev=tap0\n\
                   vpngw=".to_string()
                   + &tinc_info.vip.to_string() + "\n"
                   + "ifconfig ${dev} ${vpngw} netmask  " + netmask + "\n";

            if TincRunMode::Client == self.mode {
                let default_gateway = get_default_gateway()?.to_string();
                buf = buf
                    + "route -q -n delete -net 0.0.0.0\n\
                    route -q -n add -host " + &tinc_info.connect_to[0].ip.to_string()
                    + " -gateway " + &default_gateway + "\n"
                    + "route add -host 10.255.255.254 -interface tap0 -iface -cloning\n"
                    + "route add -net 0.0.0.0 -gateway 10.255.255.254";
            }

            buf = buf + &self.tinc_home + "tinc-report -u\n";
        }
        #[cfg(windows)]
        {
            buf = "netsh interface ipv4 set address name=\"dnet\" source=static addr=".to_string() +
                &tinc_info.vip.to_string() + " mask=" + netmask + "\r\n";

            if TincRunMode::Client == self.mode {
                let default_gateway = get_default_gateway()?.to_string();
                let vnic_index = format!("{}", get_vnic_index()?);

                buf = buf
                    + "route add " + &tinc_info.connect_to[0].ip.to_string()
                        + " mask 255.255.255.255 " + &default_gateway + "\r\n"
                    + "route add 10.255.255.254 mask 255.255.255.255 10.255.255.254 if "
                        + &vnic_index + "\r\n"
                    + "route add 0.0.0.0 mask 0.0.0.0 10.255.255.254 if "
                        + &vnic_index + "\r\n";
            }

            buf = buf + &self.tinc_home + "tinc-report.exe -u";
        }

        let path = self.tinc_home.clone() + TINC_UP_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::FileCreateError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            set_script_permissions(&path)?;
        Ok(())
    }

    fn set_tinc_down(&self, tinc_info: &TincInfo) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        let buf;
        #[cfg(target_os = "linux")]
        {
            buf = "#!/bin/bash\n".to_string() + &self.tinc_home + "tinc-report -d";
        }
        #[cfg(target_os = "macos")]
        {
            let default_gateway = get_default_gateway()?.to_string();
            buf = "#!/bin/bash\n".to_string() + &self.tinc_home + "tinc-report -d"
                + "route -n -q delete -host " + &tinc_info.connect_to[0].ip.to_string() + "\n"
                + "route -n -q delete -net 0.0.0.0 \n\
                   route -n -q add -net 0.0.0.0 -gateway " + &default_gateway;
        }
        #[cfg(windows)]
        {
            let vnic_index = format!("{}", get_vnic_index()?);
            buf = "route delete 0.0.0.0 mask 0.0.0.0 10.255.255.254 if ".to_string()
                + &vnic_index + "\r\n"
                + &self.tinc_home.to_string() + "tinc-report.exe -d";
        }

        let path = self.tinc_home.clone() + TINC_DOWN_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            set_script_permissions(&path)?;
        Ok(())
    }

    fn set_host_up(&self) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        #[cfg(windows)]
            let buf = &(self.tinc_home.to_string() + "tinc-report.exe -hu ${NODE}");
        #[cfg(unix)]
            let buf = "#!/bin/bash\n".to_string() + &self.tinc_home + "tinc-report -hu ${NODE}";

        let path = self.tinc_home.clone() + HOST_UP_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            set_script_permissions(&path)?;
        Ok(())
    }

    fn set_host_down(&self) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        #[cfg(windows)]
            let buf = &(self.tinc_home.to_string() + "tinc-report.exe -hd ${NODE}");
        #[cfg(unix)]
            let buf = "#!/bin/bash\n".to_string() + &self.tinc_home + "tinc-report -hd ${NODE}";

        let path = self.tinc_home.clone() + HOST_DOWN_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            set_script_permissions(&path)?;
        Ok(())
    }

    pub fn create_tinc_dirs(&self) -> Result<()> {
        let path_str = self.tinc_home.clone() + "hosts";
        if !std::path::Path::new(&path_str).is_dir() {
            fs::create_dir_all(&path_str)
                .map_err(|_| Error::IoError("Can not create tinc home dir".to_string()))?;
        }
        Ok(())
    }

    /// 获取子设备公钥
    pub fn get_host_pub_key(&self, host_name: &str) -> Result<String> {
        let _guard = self.mutex.lock().unwrap();
        let file_path = &(self.tinc_home.to_string() + "hosts/" + host_name);
        let mut file = fs::File::open(file_path)
            .map_err(|_| Error::FileNotExist(file_path.to_string()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|_| Error::FileNotExist(file_path.to_string()))?;
        Ok(contents)
    }

    // 写TINC_AUTH_PATH/TINC_AUTH_FILENAME(auth/auth.txt),用于tinc reporter C程序
    // TODO 去除C上报tinc上线信息流程,以及去掉auth/auth.txt.
//    fn write_auth_file(&self,
//                           server_url:  &str,
//                           info:        &TincInfo,
//    ) -> Result<()> {
//        let path = self.tinc_home.to_string() + TINC_AUTH_PATH;
//        let auth_dir = path::PathBuf::from(&(path));
//        if !path::Path::new(&auth_dir).is_dir() {
//            fs::create_dir_all(&auth_dir)
//                .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
//        }
//
//        let file_path_buf = auth_dir.join(TINC_AUTH_FILENAME);
//        let file_path = path::Path::new(&file_path_buf);
//
//        if let Some(file_str) = file_path.to_str() {
//            let path = file_str.to_string();
//            let mut file = fs::File::create(path.clone())
//                .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
//            let auth_info = AuthInfo::load(server_url, info);
//            file.write(auth_info.to_json_str().as_bytes())
//                .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
//        }
//
//        return Ok(());
//    }

}

#[cfg(unix)]
fn set_script_permissions(path: &str) -> Result<()>{
    use std::{os::unix::fs::PermissionsExt};
    fs::set_permissions(&path, PermissionsExt::from_mode(0o755))
        .map_err(Error::PermissionsError)?;
    Ok(())
}

#[cfg(windows)]
fn get_vnic_index() -> Result<u32> {
    let adapters = ipconfig::get_adapters().unwrap();
    for interface in adapters {
        if interface.friendly_name() == "dnet" {
            return Ok(interface.ipv6_if_index());
        }
    }
    Err(Error::VnicNotFind("No Adapter name \"dnet\" find".to_string()))
}

#[cfg(target_os = "macos")]
fn get_default_gateway() -> Result<IpAddr> {
    let cmd = duct::cmd!(
        "route", "-n", "get", "default"
    );
    let res = cmd.read().map_err(|e|Error::GetDefaultGatewayError(e.to_string()))?;
    let res: Vec<&str> = res.split("gateway: ").collect();
    if res.len() < 1 {
        return Err(Error::GetDefaultGatewayError("route -n get default not find gateway:".to_string()));
    }
    let res: Vec<&str> = res[1].split("\n").collect();
    let res = res[0];
    IpAddr::from_str(res).map_err(|e|Error::GetDefaultGatewayError(e.to_string()))
}

#[cfg(target_os = "windows")]
fn get_default_gateway() -> Result<IpAddr> {
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
    for i in 0..5 {
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
