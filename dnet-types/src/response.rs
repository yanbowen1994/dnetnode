pub use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub code:   u32,
    pub msg:    String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data:   Option<Value>
}

impl Response {
    pub fn new(code: u32, msg: String, data: Option<Value>) -> Self {
        Self {
            code,
            msg,
            data,
        }
    }

    pub fn set_msg(mut self, msg: String) -> Self {
        self.msg = msg;
        self
    }

    pub fn set_data(mut self, data: Option<Value>) -> Self {
        self.data = data;
        self
    }

    pub fn success() -> Self {
        Self {
            code: 200,
            msg:  "".to_owned(),
            data: None,
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            code: 401,
            msg:  "Unauthorized".to_owned(),
            data: None,
        }
    }

    pub fn user_not_exist() -> Self {
        Self {
            code: 919,
            msg: "UserNotExist".to_owned(),
            data: None,
        }
    }

    pub fn internal_error() -> Self {
        Self {
            code: 500,
            msg:  "".to_owned(),
            data: None,
        }
    }

    pub fn not_login() -> Self {
        Self {
            code:   500,
            msg:    "NotLogIn.".to_owned(),
            data:   None,
        }
    }

    pub fn exec_timeout() -> Self {
        Self {
            code: 500,
            msg:  "Timeout".to_string(),
            data: None,
        }
    }
    pub fn http_401() -> Self {
        Self {
            code: 401,
            msg: "No authentication or authentication failure".to_owned(),
            data: None,
        }
    }

    pub fn http_402() -> Self {
        Self {
            code: 402,
            msg: "The account is locked or inactive".to_owned(),
            data: None,
        }
    }

    pub fn http_403() -> Self {
        Self {
            code: 403,
            msg: "Validation code is invalid, please re-validate".to_owned(),
            data: None,
        }
    }

    pub fn http_404() -> Self {
        Self {
            code: 404,
            msg: "Not Found".to_owned(),
            data: None,
        }
    }

    pub fn http_406() -> Self {
        Self {
            code: 406,
            msg: "Failed to send mail".to_owned(),
            data: None,
        }
    }

    pub fn http_500() -> Self {
        Self {
            code: 500,
            msg: "Internal Server Error".to_owned(),
            data: None,
        }
    }

    pub fn http_800() -> Self {
        Self {
            code: 800,
            msg: "Error reading version information".to_owned(),
            data: None,
        }
    }

    pub fn http_801() -> Self {
        Self {
            code: 801,
            msg: "Version does not exist".to_owned(),
            data: None,
        }
    }

    pub fn http_802() -> Self {
        Self {
            code: 802,
            msg: "Download version error".to_owned(),
            data: None,
        }
    }

    pub fn http_900() -> Self {
        Self {
            code: 900,
            msg: "Other runtime errors".to_owned(),
            data: None,
        }
    }

    pub fn http_901() -> Self {
        Self {
            code: 901,
            msg: "Parameter error".to_owned(),
            data: None,
        }
    }

    pub fn http_902() -> Self {
        Self {
            code: 902,
            msg: "The server did not start successfully and cannot provide services.".to_owned(),
            data: None,
        }
    }

    pub fn http_903() -> Self {
        Self {
            code: 903,
            msg: "Proxy not registered".to_owned(),
            data: None,
        }
    }

    pub fn http_904() -> Self {
        Self {
            code: 904,
            msg: "Group does not exist".to_owned(),
            data: None,
        }
    }

    pub fn http_905() -> Self {
        Self {
            code: 905,
            msg: "Too many members".to_owned(),
            data: None,
        }
    }

    pub fn http_906() -> Self {
        Self {
            code: 906,
            msg: "There is no such device under the group".to_owned(),
            data: None,
        }
    }

    pub fn http_907() -> Self {
        Self {
            code: 907,
            msg: "No proxy to assign".to_owned(),
            data: None,
        }
    }

    pub fn http_908() -> Self {
        Self {
            code: 908,
            msg: "There is no such group under the user".to_owned(),
            data: None,
        }
    }

    pub fn http_909() -> Self {
        Self {
            code: 909,
            msg: "Unsupported operation".to_owned(),
            data: None,
        }
    }

    pub fn http_910() -> Self {
        Self {
            code: 910,
            msg: "Routers without authentication".to_owned(),
            data: None,
        }
    }

    pub fn http_911() -> Self {
        Self {
            code: 911,
            msg: "More than the number of routers that can be added to this group.".to_owned(),
            data: None,
        }
    }

    pub fn http_912() -> Self {
        Self {
            code: 912,
            msg: "The router is not responding".to_owned(),
            data: None,
        }
    }

    pub fn http_913() -> Self {
        Self {
            code: 913,
            msg: "No permission to delete".to_owned(),
            data: None,
        }
    }

    pub fn http_914() -> Self {
        Self {
            code: 914,
            msg: "No permission to modify".to_owned(),
            data: None,
        }
    }

    pub fn http_915() -> Self {
        Self {
            code: 915,
            msg: "No permission to add".to_owned(),
            data: None,
        }
    }

    pub fn http_916() -> Self {
        Self {
            code: 916,
            msg: "No permission to remove".to_owned(),
            data: None,
        }
    }

    pub fn http_917() -> Self {
        Self {
            code: 917,
            msg: "Device does not exist".to_owned(),
            data: None,
        }
    }

    pub fn http_918() -> Self {
        Self {
            code: 918,
            msg: "Already joined".to_owned(),
            data: None,
        }
    }

    pub fn http_919() -> Self {
        Self {
            code: 919,
            msg: "User does not exist".to_owned(),
            data: None,
        }
    }

    pub fn http_920() -> Self {
        Self {
            code: 920,
            msg: "Can't invite yourself".to_owned(),
            data: None,
        }
    }

    pub fn http_921() -> Self {
        Self {
            code: 921,
            msg: "Can't remove yourself".to_owned(),
            data: None,
        }
    }

    pub fn http_922() -> Self {
        Self {
            code: 922,
            msg: "No virtual IP can be assigned".to_owned(),
            data: None,
        }
    }

    pub fn http_923() -> Self {
        Self {
            code: 923,
            msg: "Login log not exist".to_owned(),
            data: None,
        }
    }

    pub fn http_924() -> Self {
        Self {
            code: 924,
            msg: "Update publickey fail".to_owned(),
            data: None,
        }
    }

    pub fn http_925() -> Self {
        Self {
            code: 925,
            msg: "The mac address corresponding device is not reported".to_owned(),
            data: None,
        }
    }

    pub fn http_926() -> Self {
        Self {
            code: 926,
            msg: "The device corresponding to the mac address does not have public key information".to_owned(),
            data: None,
        }
    }

    pub fn http_927() -> Self {
        Self {
            code: 927,
            msg: "Proxy does not exist".to_owned(),
            data: None,
        }
    }

    pub fn http_928() -> Self {
        Self {
            code: 928,
            msg: "Device does not exist".to_owned(),
            data: None,
        }
    }

    pub fn http_929() -> Self {
        Self {
            code: 929,
            msg: "Proxy offline".to_owned(),
            data: None,
        }
    }

    pub fn http_930() -> Self {
        Self {
            code: 930,
            msg: "The assigned IP address format is incorrect".to_owned(),
            data: None,
        }
    }

    pub fn http_931() -> Self {
        Self {
            code: 931,
            msg: "The device already exists in the group".to_owned(),
            data: None,
        }
    }

    pub fn http_932() -> Self {
        Self {
            code: 932,
            msg: "Troubleshooting failed".to_owned(),
            data: None,
        }
    }

    pub fn http_933() -> Self {
        Self {
            code: 933,
            msg: "The device is not currently joined to the group".to_owned(),
            data: None,
        }
    }

    pub fn http_934() -> Self {
        Self {
            code: 934,
            msg: "Operation database failure".to_owned(),
            data: None,
        }
    }

    pub fn http_935() -> Self {
        Self {
            code: 935,
            msg: "No router is in the team, no client device is allowed to be added".to_owned(),
            data: None,
        }
    }

    pub fn http_936() -> Self {
        Self {
            code: 936,
            msg: "The number of users exceeds the maximum number".to_owned(),
            data: None,
        }
    }

    pub fn http_937() -> Self {
        Self {
            code: 937,
            msg: "Maximum number of online users".to_owned(),
            data: None,
        }
    }

    pub fn http_938() -> Self {
        Self {
            code: 938,
            msg: "The device has been bound by other users.".to_owned(),
            data: None,
        }
    }

    pub fn http_939() -> Self {
        Self {
            code: 939,
            msg: "Username have registered".to_owned(),
            data: None,
        }
    }

    pub fn http_940() -> Self {
        Self {
            code: 940,
            msg: "Useremail have registered".to_owned(),
            data: None,
        }
    }

    pub fn http_941() -> Self {
        Self {
            code: 941,
            msg: "fail to modify password".to_owned(),
            data: None,
        }
    }

    pub fn http_942() -> Self {
        Self {
            code: 942,
            msg: "fail to modify user's information".to_owned(),
            data: None,
        }
    }

    pub fn http_943() -> Self {
        Self {
            code: 943,
            msg: "FRIENDS_ALREADY".to_owned(),
            data: None,
        }
    }

    pub fn http_944() -> Self {
        Self {
            code: 944,
            msg: "The proxy has been bound by other users.".to_owned(),
            data: None,
        }
    }

    pub fn http_945() -> Self {
        Self {
            code: 945,
            msg: "The vpn groud disabled.".to_owned(),
            data: None,
        }
    }

    pub fn http_946() -> Self {
        Self {
            code: 946,
            msg: "Online proxy cannot be deleted".to_owned(),
            data: None,
        }
    }

    pub fn new_from_code(code: i32) -> Self {
        let msg = match code {
            401 => "No authentication or authentication failure".to_owned(),
            402 => "The account is locked or inactive".to_owned(),
            403 => "Validation code is invalid, please re-validate".to_owned(),
            404 => "Not Found".to_owned(),
            406 => "Failed to send mail".to_owned(),
            500 => "Internal Server Error".to_owned(),
            800 => "Error reading version information".to_owned(),
            801 => "Version does not exist".to_owned(),
            802 => "Download version error".to_owned(),
            900 => "Other runtime errors".to_owned(),
            901 => "Parameter error".to_owned(),
            902 => "The server did not start successfully and cannot provide services.".to_owned(),
            903 => "Proxy not registered".to_owned(),
            904 => "Group does not exist".to_owned(),
            905 => "Too many members".to_owned(),
            906 => "There is no such device under the group".to_owned(),
            907 => "No proxy to assign".to_owned(),
            908 => "There is no such group under the user".to_owned(),
            909 => "Unsupported operation".to_owned(),
            910 => "Routers without authentication".to_owned(),
            911 => "More than the number of routers that can be added to this group.".to_owned(),
            912 => "The router is not responding".to_owned(),
            913 => "No permission to delete".to_owned(),
            914 => "No permission to modify".to_owned(),
            915 => "No permission to add".to_owned(),
            916 => "No permission to remove".to_owned(),
            917 => "Device does not exist".to_owned(),
            918 => "Already joined".to_owned(),
            919 => "User does not exist".to_owned(),
            920 => "Can't invite yourself".to_owned(),
            921 => "Can't remove yourself".to_owned(),
            922 => "No virtual IP can be assigned".to_owned(),
            923 => "Login log not exist".to_owned(),
            924 => "Update publickey fail".to_owned(),
            925 => "The mac address corresponding device is not reported".to_owned(),
            926 => "The device corresponding to the mac address does not have public key information".to_owned(),
            927 => "Proxy does not exist".to_owned(),
            928 => "Device does not exist".to_owned(),
            929 => "Proxy offline".to_owned(),
            930 => "The assigned IP address format is incorrect".to_owned(),
            931 => "The device already exists in the group".to_owned(),
            932 => "Troubleshooting failed".to_owned(),
            933 => "The device is not currently joined to the group".to_owned(),
            934 => "Operation database failure".to_owned(),
            935 => "No router is in the team, no client device is allowed to be added".to_owned(),
            936 => "The number of users exceeds the maximum number".to_owned(),
            937 => "Maximum number of online users".to_owned(),
            938 => "The device has been bound by other users.".to_owned(),
            939 => "Username have registered".to_owned(),
            940 => "Useremail have registered".to_owned(),
            941 => "fail to modify password".to_owned(),
            942 => "fail to modify user's information".to_owned(),
            943 => "FRIENDS_ALREADY".to_owned(),
            944 => "The proxy has been bound by other users.".to_owned(),
            945 => "The vpn groud disabled.".to_owned(),
            946 => "Online proxy cannot be deleted".to_owned(),
            _ => "".to_owned(),
        };

        let code = if code < 0 {
            0
        }
        else {
            code as u32
        };

        Self {
            code,
            msg,
            data: None,
        }
    }
}