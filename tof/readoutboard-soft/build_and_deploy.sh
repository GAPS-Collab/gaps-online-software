#! /bin/sh

#CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic link-args=-no-pie" cross build --bin rb-soft --target=armv7-unknown-linux-gnueabi & scp target/armv7-unknown-linux-gnueabi/debug/rb-soft ucla-rb51: 
# first delete everything, since there might be remains of a previously issued cargo check
rm -rf target/armv7-unknown*
CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin rb-soft --target=armv7-unknown-linux-musleabi && scp target/armv7-unknown-linux-musleabi/debug/rb-soft ucla-rb51: 
#CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin watch-buffer-fill --target=armv7-unknown-linux-gnueabi && scp target/armv7-unknown-linux-gnueabi/debug/watch-buffer-fill ucla-rb51: 

