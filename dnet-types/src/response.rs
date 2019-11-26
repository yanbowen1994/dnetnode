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

    pub fn new_from_code(code: i32) -> Self {
        let msg = Self::get_msg_by_code(code);

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

    pub fn get_msg_by_code(code: i32) -> String {
        let msg = match code {
            401 => "Not yet logged in or the token has expired.".to_owned(),
            403 => "No relevant permissions.".to_owned(),
            405 => "Parameter value verification failed.".to_owned(),
            406 => "Parameter type failed.".to_owned(),
            407 => "Request parameter error.".to_owned(),
            408 => "file does not exist!".to_owned(),
            409 => "Data does not exist!".to_owned(),
            410 => "Username or password error. Continuous login %s times error account will lock %s minutes, you can also try %s login.".to_owned(),
            411 => "User does not exist.".to_owned(),
            412 => "User has logged out.".to_owned(),
            413 => "User has been frozen.".to_owned(),
            414 => "User is not activated.".to_owned(),
            415 => "Password error.".to_owned(),
            416 => "User logins are too many, please wait for %s seconds and try again.".to_owned(),
            417 => "User already exists！".to_owned(),
            418 => "Delete user cannot delete themselves.".to_owned(),
            419 => "No permission to delete super administrator.".to_owned(),
            441 => "Parameter cannot be empty.".to_owned(),
            600 => "The proxy failed to locate and could not obtain the IP address of the external network.".to_owned(),
            601 => "Proxy does not exist, please re-select.".to_owned(),
            602 => "Uploading public key failed. ".to_owned(),
            603 => "Update device proxy tinc status failed.".to_owned(),
            611 => "The device has been bound by another account. Please untie it first.".to_owned(),
            612 => "Device does not exist.".to_owned(),
            620 => "Router does not exist.".to_owned(),
            621 => "Router is disabled.".to_owned(),
            622 => "Router password error.".to_owned(),
            623 => "Router not bound account.".to_owned(),
            624 => "Please bind the router with your account first.!".to_owned(),
            900 => "Exception handling.".to_owned(),
            910 => "The name of the key value is not in the parameter value, please check.".to_owned(),
            911 => "The name of the key value is not in the parameter object. Please check if the value of the object is consistent.".to_owned(),

            402 => "The account is locked or inactive".to_owned(),
            404 => "Not Found".to_owned(),
            500 => "Internal Server Error".to_owned(),
            502 => "Bad Gateway".to_owned(),
            800 => "Error reading version information".to_owned(),
            801 => "Version does not exist".to_owned(),
            802 => "Download version error".to_owned(),
            901 => "Parameter error".to_owned(),
            902 => "The server did not start successfully and cannot provide services.".to_owned(),
            903 => "Proxy not registered".to_owned(),
            904 => "Group does not exist".to_owned(),
            905 => "Too many members".to_owned(),
            906 => "There is no such device under the group".to_owned(),
            907 => "No proxy to assign".to_owned(),
            908 => "There is no such group under the user".to_owned(),
            909 => "Unsupported operation".to_owned(),
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

            511 => "No usable proxy.".to_owned(),
            _ => "".to_owned(),
        };
        msg
    }
}