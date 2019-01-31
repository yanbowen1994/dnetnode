# ovrouter_Rust
### Prerequisites:
    tinc:
        liblzo2-2
    ovrouter:
        libcurl4-openssl-dev
        
### debug
    cargo run --bin ovrouter -- -d 2

### run
    Only support ubuntu 16.04
    需要在运行目录有Settings.toml文件，或者修改settings.rs，Settings.toml文件地址
    cargo build --release
    ./ovrouter -d 2
    
### todo
    Ovroute web server 
        1. uid检查(未确定Conductor发送格式)
        2. get check pub_key 返回http格式未确认
        3. 响应Conductor发送的添加hosts请求，
            1)未确定Conductor发送格式
            2)添加tinc hosts文件，在tinc operater中添加了add_hosts(未与Conductor调试)