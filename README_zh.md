# DNet proxy and client
## 平台
### Linux  
ubuntu16.04 ubuntu18.04 测试. 其他PC Linux平台理论可行  
openwrt libc采用musl, libopenssl为对应架构, 对应libc下编译并使用静态连接, 已测试armv7架构
### windows
 win7, win10已测试
### macos
 macos10+

# proxy
## 编译说明
1. 安装rust编译环境
2. 修改./env.sh openssl目录和是否静态链接openssl  
3. cargo build --release --package proxy-daemon --bin dnetovr

## 生成ubuntu下.deb安装包
[配置文件](./service_script/README.md)  
文件结构
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

## 安装说明
### 安装
```
sudo dpkg -i dnetovr.deb
```
### 删除
```
sudo dpkg -P dnetovr
```
## 配置文件说明
```toml
[tinc]
#tinc 配置文件目录
home_path = "/root/tinc/"

[server]
#conductor url 默认https
url = "test.insight.netgear.com"
#geo信息服务器url, 不可为空
geo_url = "http://52.25.79.82:10000/geoip_json.php"

[client]
#debug等级[error, warn, info, debug, trace]
log_level = "debug"
#log文件存储地址, 为空时默认"/var/log/dnetovr/"
log_dir = "/var/log/dnetovr/"
#proxy连接到conductor的用户名,密码
username = "admin"
password = "password"
#本地服务监听端口, 修改时需要与conductor同步修改
local_port = "443"
#本地服务证书文件
local_https_server_certificate_file = "/root/dnetovr/cert.pem"
#本地服务密钥文件
local_https_server_privkey_file = "/root/dnetovr/key.pem"
#代理类型
proxy_type = "other"
```

### 启动服务
```
sudo service dnetovr start
```
### 查看服务状态
```
sudo service dnetovr status
```
### 关闭服务
```
sudo service dnetovr stop
```