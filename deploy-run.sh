#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly PROJECT_NAME=zeevonk

readonly TARGET_HOST=pi@raspberrypi
readonly TARGET_PATH=/home/pi/rust/${PROJECT_NAME}
#readonly TARGET_ARCH=aarch64-unknown-linux-gnu
#readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/${PROJECT_NAME}
readonly SOURCE_PATH=.

#cargo build --release --target=${TARGET_ARCH}
#rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
#ssh -t ${TARGET_HOST} ${TARGET_PATH}

rsync -r ${SOURCE_PATH}/src/       ${TARGET_HOST}:${TARGET_PATH}/src/
rsync -r ${SOURCE_PATH}/Cargo.toml ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} "cd ${TARGET_PATH} && cargo build --release && sudo ./target/release/zeevonk"
