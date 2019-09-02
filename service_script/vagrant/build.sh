#!/bin/bash
tinc="/root/tinc"
lib="/root/tinc/lib"
git_tinc="/root/gitlab-tinc1.1"
package_path="/root"

cd /root
if [ ! -d "/root/tinc" ] ; then
	mkdir /root/tinc
	echo "mkdir /root/tinc"
fi

if [ ! -d "$lib" ] ; then
	mkdir $lib
	echo "mkdir $lib"
fi

echo "Start to check Autotools..."
ret=`find /usr/bin -name autoconf`
if [ "$ret" = "" ]; then
	sudo apt install -y autoconf
fi
ret=`find /usr/bin -name automake`
if [ "$ret" = "" ]; then
	sudo apt install -y automake
	automake --add-missing
fi
echo "Autotools...OK"


echo "Start to check OpenSSL library..."
ret1=`sudo openssl version | sed -n 1p | cut -d ' ' -f 2`
ret2=`find /root/openssl -name libcrypto.so.1.1 2>1`

echo $ret1 $ret2
if [ "$ret1" = "1.1.1c" ] && [ "$ret2" != "" ]; then
	echo "OpenSSL version: $ret1... OK"
else
	echo -e "OpenSSL version: $ret1 - To install 1.1.1c...\nStart to check git..."
	ret=`git --version 2>1 | sed -n 1p | cut -d ' ' -f 1`
	if [ "$ret" = "git" ]; then
		echo "Check git... ok"
	else
		echo "Check git - To install git..."
		sudo apt-get install -y git
		echo "Install git - ok"
	fi
	git clone https://github.com/13136106206/openssl-1.1.1c.git /root/openssl
	cd /root/openssl
	chmod 777 config
	./config shared
	make install
	sudo echo "/usr/local/lib" >> /etc/ld.so.conf
	sudo /sbin/ldconfig

	cd ..
fi

if [ ! -f "$lib/libcrypto.so.1.1" ]; then
	ret=`find /root/openssl -name libcrypto.so.1.1 2>1`
	cp $ret $lib
	echo "cp $ret $lib"
fi

echo "Start to check Lzop library..."

sudo apt-get install -y liblzo2-2 liblzo2-dev

if [ ! -f "$lib/liblzo2.so.2" ];then
	ret=`find / -name liblzo2.so.2 2>1`
	cp $ret $lib
	echo "cp $ret $lib"
fi

echo "Start to check Zlib header file..."
ret=`find /usr/include -name zlib.h 2>1`

if [ "$ret" = "" ]; then
	echo "Check Lzop header file... To install zlib"
	sudo apt-get  install -y zlib1g zlib1g-dev
	echo "Install zlib - ok"
else
	echo "Check zlib header file... OK"
fi


echo "Start to check Curses library..."
ret=`find / -name libncurses.so.5 2>1 | grep x86_64-linux-gnu | sed -n 1p`
if [ "$ret" = "" ]; then
        echo "Check Curses library... To install Curses"
        sudo apt-get install -y libncurses5
        echo "Install Curses - ok"
else
        echo "Check Curses library... OK"
fi

ret=`find / -name curses.h 2>1 | grep /usr/include | sed -n 1p`

if [ "$ret" = "" ]; then
        echo "Check Curses dev-library... To install Curses"
        sudo apt-get install -y libncurses5-dev
        echo "Install Curses - ok"
else
        echo "Check Curses library... OK"
fi

if [ ! -f "$lib/libncurses.so.5" ]; then
        ret=`find / -name libncurses.so.5 2>1 | grep x86_64-linux-gnu | sed -n 1p`
        cp $ret $lib
        echo "cp $ret $lib"
fi



echo "Start to check Readline library..."
ret=`find / -name libreadline.so.8 2>1`
if [ "$ret" = "" ]; then
	echo "Check Readline library... To install Readline library"
	wget http://ftp.gnu.org/gnu/readline/readline-8.0.tar.gz
	tar -zxvf readline-8.0.tar.gz
	cd readline-8.0
	chmod 777 configure
	./configure --enable-shared
	make
	sudo make install
	cd..
else
	echo "Check Readline library... OK"
fi

if [ ! -f "$lib/libreadline.so.8" ]; then
	ret=`find / -name libreadline.so.8 2>1`
	cp $ret $lib
	echo "cp $ret $lib"
fi

echo "Update tinc..."

if [ ! -f "$git_tinc/src/tincd" ]; then
	git clone -b 1.1 http://haungjue.deng:SiteView123@gitlab.dnetlab.com/dnet/tinc.git $git_tinc
	cd $git_tinc
	git pull
	autoreconf -fsi
	chmod 777 configure
	./configure --with-openssl-lib=/root/openssl/ --with-openssl-include=/root/openssl/include
else
	cd $git_tinc
	git pull
fi

echo "Update tinc... OK"


echo "Compile tinc..."

time1=`ls --full-time src/tinc | cut -d ' ' -f 7`

ret=`cat src/Makefile |grep rpath=/root/tinc/lib`
if [ "$ret" = "" ]; then
	sed -i "s#FLAGS = -g -O2 -Wall#FLAGS = -g -O2 -Wall -Wl,-rpath=/root/tinc/lib#g" src/Makefile
fi
make
sudo make install

time2=`ls --full-time src/tinc | cut -d ' ' -f 7`
if [ "$time2" = "$time1" ]; then
	echo Compiler failed !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	exit 1
fi

echo "Compile tinc... ... ... ... ... ... successfully!"

echo "start deploying the cargo environment"

sudo -s
su
cd /root
wget -O rustup.sh https://sh.rustup.rs
chmod 0755 rustup.sh
./rustup.sh -y

while [ ! -f "cargo" ];do
    ret=`/root/.cargo/bin/cargo --version | cut -d ' ' -f 2`
    if [ "$ret" == "" ]; then
        /root/rustup.sh -y
    else
        break;
    fi;
done

export OPENSSL_DIR="/usr/local"
export OPENSSL_STATIC="1"

echo "git clone ovrouter"
git clone http://bowen.yan:siteview123%21%40%23@gitlab.dnetlab.com/dnet/ovrouter_netgear.git
cd ovrouter_netgear

echo "cargo build"
while [ ! -f "./target/release/dnetovr" ]; do
    /root/.cargo/bin/cargo build --release
done
echo "cargo build finsh"

cd /root

mkdir -p /root/dnetovr/DEBIAN /root/dnetovr/lib/systemd/system /root/dnetovr/root/dnetovr /root/dnetovr/root/tinc/lib

cp /root/ovrouter_netgear/cert.pem ./dnetovr/root/dnetovr
cp /root/ovrouter_netgear/key.pem ./dnetovr/root/dnetovr
cp /root/ovrouter_netgear/settings.toml.example ./dnetovr/root/dnetovr/settings.toml
cp /root/ovrouter_netgear/target/release/dnetovr ./dnetovr/root/dnetovr

cp /root/ovrouter_netgear/service_script/control  ./dnetovr/DEBIAN
cp /root/ovrouter_netgear/service_script/dnetovr.service  ./dnetovr/lib/systemd/system/dnetovr.service

cp /root/tinc /root/dnetovr/root

dpkg-deb -b /root/dnetovr
echo "All build finsh"
