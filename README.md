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