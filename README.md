# DNetovr NETGEAR Version
[中文说明](./README_zh.md)
## platform
Linux  
Ubuntu16.04 ubuntu18.04 to complete the test. Other PC Linux should work.

## Compilation instructions
1. Install the rust build environment
2. Modify openssl directory and whether statically link openssl in the ./env.sh.
3. cargo build --release

## Generate a .deb installation package under ubuntu
[config files](./service_script/README.md)    
File tree
```
.
├── DEBIAN
│   └── control
├── lib
│   └── systemd
│       └── system
│           └── dnetovr.service
└── root
    ├── dnetovr
    │   ├── cert.pem
    │   ├── dnetovr
    │   ├── key.pem
    │   └── settings.toml
    └── tinc
        ├── lib
        │   ├── libcrypto.so.1.1
        │   ├── liblzo2.so.2
        │   ├── libncurses.so.5
        │   ├── libreadline.so.8 -> ./libreadline.so.8.0
        │   ├── libreadline.so.8.0
        │   ├── libssl.so.1.1
        │   ├── libtinfo.so.5
        │   └── libz.so.1
        ├── proxyReport
        ├── tinc
        └── tincd
```

## Install Notes
### Install
```
sudo dpkg -i dnetovr.deb
```
### Remove
```
sudo dpkg -P dnetovr
```
## Config File Notes
```toml
[tinc]
#Tinc config directory
home_path = "/root/tinc/"

[server]
#Conductor url, default https
url = "test.insight.netgear.com"
#Geo server url, not nullable
geo_url = "http://52.25.79.82:10000/geoip_json.php"

[client]
#Debug level [error, warn, info, debug, trace]
log_level = "debug"
#Log dir storage address, default is "/var/log/dnetovr/".
log_dir = "/var/log/dnetovr/"
#Username and password for the proxy connection to the conductor
username = "admin"
password = "password"
#Local service listening port. Need to be modified synchronously with conductor.
local_port = "443"
#Local service certificate file
local_https_server_certificate_file = "/root/dnetovr/cert.pem"
#Local service priv_key file
local_https_server_privkey_file = "/root/dnetovr/key.pem"
#set proxy type
proxy_type = "other"
```

### Start dnetovr service 
```
sudo service dnetovr start
```
### Check dnetovr service status 
```
sudo service dnetovr status
```
### stop dnetovr service
```
sudo service dnetovr stop
```
