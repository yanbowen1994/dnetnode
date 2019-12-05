use std::sync::Mutex;

use crate::TincRunMode;

static mut EL: *mut TincOperator = 0 as *mut _;

pub struct TincSettings {
    pub tinc_home:                          String,
    pub mode:                               TincRunMode,
    pub port:                               u16,
    pub tinc_memory_limit:                  f64,
    pub tinc_allowed_out_memory_times:      u32,
    pub tinc_allowed_tcp_failed_times:      u32,
    pub tinc_check_frequency:               u32,
    pub external_boot:                      bool,
}

impl Default for TincSettings {
    fn default() -> Self {
        Self {
            tinc_home:                          "/opt/dnet/tinc/".to_owned(),
            mode:                               TincRunMode::Client,
            port:                               50069,
            tinc_memory_limit:                  100.0,
            tinc_allowed_out_memory_times:      0,
            tinc_allowed_tcp_failed_times:      0,
            tinc_check_frequency:               0,
            external_boot:                      false,
        }
    }
}

/// Tinc operator
pub struct TincOperator {
    pub mutex:                  Mutex<i32>,
    pub tinc_settings:          TincSettings,
    pub tinc_out_memory_times:  u32,
}

impl TincOperator {
    /// 获取tinc home dir 创建tinc操作。
    pub fn new(tinc_settings: TincSettings) {
        let operator = TincOperator {
            mutex:                  Mutex::new(0),
            tinc_out_memory_times:  0,
            tinc_settings,
        };

        operator.clear_hosts();

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
}