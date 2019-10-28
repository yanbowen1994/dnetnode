#!/bin/sh /etc/rc.common
START=99

start() {
    /usr/local/opt/dnet/dnet-daemon -d 0 -c /usr/local/opt/dnet/
}

stop() {
    killall dnet-daemon
}