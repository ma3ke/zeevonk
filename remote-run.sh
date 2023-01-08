#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly PROJECT_NAME=zeevonk

readonly TARGET_HOST=pi@raspberrypi
readonly TARGET_PATH=/home/pi/rust/${PROJECT_NAME}

ssh -t ${TARGET_HOST} "cd ${TARGET_PATH} && sudo ./target/release/zeevonk"
