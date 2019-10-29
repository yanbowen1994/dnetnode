#!/bin/sh /etc/rc.common
START=99

start() {
    /usr/local/opt/dnet/dnet-daemon -c /usr/local/opt/dnet/
}

stop() {
    killall dnet-daemon
}