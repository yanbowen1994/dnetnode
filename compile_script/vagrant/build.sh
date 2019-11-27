#!/bin/bash
sudo apt update
sudo apt install -y cmake autoconf automake git liblzo2-2 liblzo2-dev zlib1g zlib1g-dev libncurses5 libncurses5-dev
chmod 0755 /mnt/build.py
sudo -s
su
/mnt/build.py init