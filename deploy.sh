#!/bin/bash

# Thanks to https://medium.com/swlh/compiling-rust-for-raspberry-pi-arm-922b55dbb050 for this nice script.

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST="$1"
readonly TARGET_PATH=/home/pi/zeevonk
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/zeevonk

cross build --target=armv7-unknown-linux-gnueabihf --release --bin zeevonk
rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} "sudo ${TARGET_PATH}"
