#!/bin/bash
set -eux

curPath=$(readlink -f "$(dirname "$0")")
cd $curPath

source ../../env.sh
cargo build --release --target=arm-unknown-linux-musleabihf --package=dnet-daemon
cargo build --release --target=arm-unknown-linux-musleabihf --package=dnet-cgi
cargo build --release --target=arm-unknown-linux-musleabihf --package=tinc-plugin
cp ../../target/arm-unknown-linux-musleabihf/release/dnet-daemon ./dnet/dnet/
cp ../../target/arm-unknown-linux-musleabihf/release/dnet-cgi ./dnet/dnet/dnet-cgi
cp ../../target/arm-unknown-linux-musleabihf/release/tinc-report ./dnet/dnet/tinc/tinc-report

cp ../../settings.toml.router_example ./dnet/dnet/settings.toml

tar cvf dnet.tar ./dnet/

