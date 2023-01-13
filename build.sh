TARGET_CFLAGS='-mfpu=neon-vfpv4 -mmusl -mcpu=cortex-a53'
cargo build --target armv7-unknown-linux-musleabihf --release
