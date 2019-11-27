#!/usr/bin/python3
import os
import sys

tinc_lib_dir = "/root/tinc/lib"
try:
    os.system("mkdir -p " + tinc_lib_dir)
except:
    pass

if len(sys.argv) > 1:
    os.system("cp /usr/lib/x86_64-linux-gnu/liblzo2.so " + tinc_lib_dir + "/liblzo2.so.2")
    os.system("cp /usr/lib/x86_64-linux-gnu/libncurses.so " + tinc_lib_dir + "/libncurses.so.5")

    openssl_dir = "/root/openssl"
    while True:
        try:
            os.system("git clone --branch OpenSSL_1_1_1c"
                      " https://github.com/openssl/openssl.git " + openssl_dir)
            os.system("cd " + openssl_dir)
            os.system("chmod 777 config")
            os.system("./config shared")
            os.system("make")
            os.system("cp " + openssl_dir + "/libcrypto.so.1.1 " + tinc_lib_dir)
            break
        except:
            pass

    fec_dir = "/root/libmyfec"
    os.system("cd /root")
    os.system("git clone http://bowen.yan:siteview123%21%40%23@git.dnetlab.com/dnet/libmyfec.git "
              + fec_dir)
    os.system("cd " + fec_dir)
    os.system("mkdir build")
    os.system("cd build")
    os.system("cmake ..")
    os.system("make")
    os.system("sudo make install")
    os.system("cp /usr/local/myfec/include /usr/include")
    os.system("cp /usr/local/lib/libmyfec.so /usr/lib")
    os.system("cp /usr/local/lib/libmyfec.so /root/tinc/lib")

    readline_dir = "/root/readline-8.0"
    os.system("cd /root")
    os.system("wget http://ftp.gnu.org/gnu/readline/readline-8.0.tar.gz")
    os.system("tar -zxvf readline-8.0.tar.gz")
    os.system("cd " + readline_dir)
    os.system("chmod 777 configure")
    os.system("./configure --enable-shared")
    os.system("make")
    os.system("sudo make install")
    os.system("cp /usr/lib/x86_64-linux-gnu/libreadline.so " + tinc_lib_dir + "/libreadline.so.8")

    cargo_dir = "/root/.cargo"
    os.system("cd /root")
    while True:
        try:
            os.system("wget -O rustup.sh https://sh.rustup.rs")
            os.system("chmod 0755 rustup.sh")
            os.system("./rustup.sh -y")
            break
        except:
            pass


tinc_dir = "/root/tinc_src"
if not os.path.exists(tinc_dir):
    os.system("cd /root")
    os.system("git clone -b 1.2 http://bowen.yan:siteview123%21%40%23@git.dnetlab.com/dnet/tinc.git "
         + tinc_dir)
    os.system("cd " + tinc_dir)
else:
    os.system("cd " + tinc_dir)
    os.system("git pull")

os.system("cd " + tinc_dir)
os.system("autoreconf -fsi")
os.system("chmod 777 configure")
os.system("./configure --with-openssl-lib=/root/openssl/"
          " --with-openssl-include=/root/openssl/include"
          " --with-readline-lib=/root/readline-8.0/shlib/"
          " --with-readline-include=/root/readline-8.0/include")
os.system("make")

dnet_dir = "/root/dnetnode"
if not os.path.exists(dnet_dir):
    os.system("git clone http://bowen.yan:siteview123%21%40%23@git.dnetlab.com/dnet/dnetnode "
              + dnet_dir)
    os.system("cd " + dnet_dir)
else:
    os.system("cd " + dnet_dir)
    os.system("git pull --rebase")

os.system("export PATH='$HOME/.cargo/bin:$PATH'")
os.putenv("OPENSSL_DIR", "/usr/local")
os.putenv("OPENSSL_STATIC", "1")
os.system("cd " + dnet_dir)
os.system("/root/.cargo/bin/cargo build --release")

os.system("cd /root")
os.system("mkdir -p /root/dnetovr/DEBIAN /root/dnetovr/lib/systemd/system "
          "/root/dnetovr/root/dnetovr /root/dnetovr/root/tinc/lib")
os.system("cp /root/dnetnode/cert.pem ./dnetovr/root/dnetovr")
os.system("cp /root/dnetnode/key.pem ./dnetovr/root/dnetovr")
os.system("cp /root/dnetnode/settings.toml.example ./dnetovr/root/dnetovr/settings.toml")
os.system("cp /root/dnetnode/target/release/dnetovr ./dnetovr/root/dnetovr")
os.system("cp /root/dnetnode/compile_script/control  ./dnetovr/DEBIAN")
os.system("cp /root/dnetnode/compile_script/dnetovr.service  ./dnetovr/lib/systemd/system/dnetovr.service")
os.system("cp /root/tinc /root/dnetovr/root -rf")
os.system("dpkg-deb -b /root/dnetovr")
os.system("cp /root/dnetovr.deb /mnt/dnetovr")



