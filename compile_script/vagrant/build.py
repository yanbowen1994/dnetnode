#!/usr/bin/python3
import os
import sys

tinc_lib_dir = "/root/tinc/lib"
try:
    os.system("mkdir -p " + tinc_lib_dir)
except:
    pass

if len(sys.argv) > 1 and sys.argv[1] == "init":
    os.system("cp /usr/lib/x86_64-linux-gnu/liblzo2.so " + tinc_lib_dir + "/liblzo2.so.2")
    os.system("cp /lib/x86_64-linux-gnu/libncurses.so.5.9 " + tinc_lib_dir + "/libncurses.so.5")

    openssl_dir = "/root/openssl"

    print("download openssl")

    os.system("git clone -b OpenSSL_1_1_1c https://github.com/openssl/openssl.git " + openssl_dir)
    print("download openssl finish.")

    os.chdir(openssl_dir)
    os.system("chmod 777 config")
    os.system("./config shared")
    os.system("make install")

    readline_dir = "/root/readline-8.0"
    os.chdir("/root")
    os.system("wget http://ftp.gnu.org/gnu/readline/readline-8.0.tar.gz")
    os.system("tar -zxvf readline-8.0.tar.gz")
    os.chdir(readline_dir)
    os.system("chmod 777 configure")
    os.system("./configure --enable-shared")
    os.system("make")
    os.system("sudo make install")
    os.system("cp /usr/lib/x86_64-linux-gnu/libreadline.so " + tinc_lib_dir + "/libreadline.so.8")

    cargo_dir = "/root/.cargo"
    os.chdir("/root")
    while True:
        try:
            os.system("wget -O rustup.sh https://sh.rustup.rs")
            os.system("chmod 0755 rustup.sh")
            os.system("./rustup.sh -y")
            break
        except:
            pass

if len(sys.argv) > 1 and (sys.argv[1] == "tinc" or sys.argv[1] == "init"):
    tinc_dir = "/root/tinc_src"
    if not os.path.exists(tinc_dir):
        os.chdir("/root")
        os.system("git clone -b Release-1.1pre17 https://bowen.yan:siteview123%21%40%23@git.vlan.cn/DNET/tinc.git "
             + tinc_dir)
        os.chdir(tinc_dir)
    else:
        os.chdir(tinc_dir)

    os.chdir(tinc_dir)
    os.system("autoreconf -fsi")
    os.system("chmod 777 configure")
    os.system("./configure"
              " --with-readline-lib=/root/readline-8.0/shlib/"
              " --with-readline-include=/root/readline-8.0/include")
    os.system("sed -i 's#FLAGS = -g -O2 -Wall#FLAGS = -g -O2 -Wall -Wl,-rpath=/opt/dnet/tinc/lib#g' "
              + tinc_dir + "/src/Makefile")
    os.system("make")
    if os.system("cp " + tinc_dir + "/src/tinc /root/tinc/tinc"):
        print("compile tinc failed.")
        exit(1)
    if os.system("cp " + tinc_dir + "/src/tincd /root/tinc/tincd"):
        print("compile tincd failed.")
        exit(1)


if len(sys.argv) == 1 \
        or (len(sys.argv) > 1
            and (sys.argv[1] == "dnet" or sys.argv[1] == "init")):
    dnet_dir = "/root/dnetnode"
    if not os.path.exists(dnet_dir):
        os.system("git clone -b origin_tinc http://bowen.yan:siteview123%21%40%23@git.vlan.cn/dnet/dnetnode "
                  + dnet_dir)
        os.chdir(dnet_dir)
    else:
        os.chdir(dnet_dir)
        os.system("git pull --rebase")

    path = os.getenv("PATH")
    path += ":$HOME/.cargo/bin"
    os.putenv("PATH", path)
    os.putenv("OPENSSL_DIR", "/usr/local")
    os.putenv("OPENSSL_STATIC", "1")
    os.chdir(dnet_dir)
    os.system("/root/.cargo/bin/rustup update")
    os.system("/root/.cargo/bin/cargo build --release")

os.chdir("/root")
build_dir = "/root/dnet"
os.system("mkdir -p /root/dnet/DEBIAN /root/dnet/lib/systemd/system "
          "/root/dnet/opt/dnet /root/dnet/opt/dnet/tinc/lib "
          "/root/dnet/opt/dnet/tinc")
os.system("cp /root/dnetnode/cert.pem ./dnet/opt/dnet")
os.system("cp /root/dnetnode/key.pem ./dnet/opt/dnet")
os.system("cp /root/dnetnode/settings.example.toml ./dnet/opt/dnet/settings.example.toml")
os.system("cp /root/dnetnode/target/release/dnet-daemon ./dnet/opt/dnet")
os.system("cp /root/dnetnode/target/release/dnet ./dnet/opt/dnet")
os.system("cp /root/dnetnode/target/release/tinc-report ./dnet/opt/dnet/tinc")
os.system("cp /root/dnetnode/compile_script/control ./dnet/DEBIAN")
os.system("cp /root/dnetnode/compile_script/dnet.service ./dnet/lib/systemd/system/dnet.service")
os.system("cp /root/tinc/* /root/dnet/opt/dnet/tinc -rf")
os.system("dpkg-deb -b /root/dnet dnet.deb")
os.system("cp /root/dnet.deb /mnt/")
print("finish")
