# 2019/12/17 测试
## 路由器操作
1. 固件更新
2. telnet 到路由器修改conductor地址
    vi /usr/local/opt/dnet/settings.toml
    conductor_url = "http://58.20.63.22"
3. 更新固件后, 手动重启一次路由器

## 到网站添加设备(管理员执行)
1. 登录 http://58.20.63.22:7075 (具体流程咨询前端)
    账号: superadmin  
    密码: password
2. 添加设备
    通过路由器序列号, 将该路由器注册为组网路由器(该操作为系统管理员执行,且不会绑定设备到用户)

## 用户绑定设备
1. 登录 http://58.20.63.22:7075 (具体流程咨询前端)
    测试时, 可以使用superadmin, 也应该可以自己注册账号
    账号: superadmin  
    密码: password
2. 绑定路由器到当前用户
3. 创建组
4. 将设备添加到组

## 测试
1. 检查进程
    dnet-daemon 应该在路由器启动时自动启动
    ps|grep dnet-daemon 查看是否有dnet-daemon 进程
    
    如果网站绑定成功, 路由器tincd 应当自启动,
    ps|grep tincd 查看是否有tincd进程
    
2. 查看路由器连接状态
    查看该路由器设备连接状态(建议使用linux客户端查看)

3. 确认应该连接到那些设备后, 查看路由表
    结果为 host名, 是否在路由表中

4. 确认应该连接到那些设备后, ping其他设备
    结果为 host名, 是否可ping通
    
## 高级测试
1. 关闭自启动的dnet-daemon
    ```
    killall dnet-daemon
    ```
2. 关闭tincd
    ```
    killall tincd
    ```
3. debug 模式启动dnet-daemon
    ```
    cd /usr/local/opt/dnet
    ./dnet-daemon -c . -d 2 
    ```

4. 通过arm下的dnet-cli命令行工具调试  
    固件中没有该工具, 需要额外传入路由器用于调试.  
    该工具使用方法类似linux下的dnet, 但是name改为dnet-cli, 以防与cgi工具dnet冲突  
    ./dnet-cli status
    ./dnet-cli group list
    以上状态查看命令有效
    ./dnet-cli login logout 不应适用
