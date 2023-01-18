#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly PROJECT_NAME=zeevonk

readonly TARGET_HOST=pi@raspberrypi
readonly TARGET_PATH=/home/pi/rust/${PROJECT_NAME}
readonly SOURCE_PATH=.

rsync -a ${SOURCE_PATH}/src/       ${TARGET_HOST}:${TARGET_PATH}/src/
rsync -a ${SOURCE_PATH}/Cargo.toml ${TARGET_HOST}:${TARGET_PATH}/
ssh -t ${TARGET_HOST} "cd ${TARGET_PATH} && cargo build --release && sudo ./target/release/zeevonk"
