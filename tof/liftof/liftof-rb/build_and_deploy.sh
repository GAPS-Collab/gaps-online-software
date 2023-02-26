#! /bin/sh

compile_and_deploy_target() {
  CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin $1 --target=armv7-unknown-linux-musleabi --release && scp ../target/armv7-unknown-linux-musleabi/release/$1 ucla-rb102: 
}

# first delete everything, since there might be remains of a previously issued cargo check
rm -rf ../target/armv7-unknown*

compile_and_deploy_target liftof-rb
#compile_and_deploy_target debug-idle
#compile_and_deploy_target watch-buffer-fill

#CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin scan-uio-buffers --target=armv7-unknown-linux-musleabi --release && scp ../target/armv7-unknown-linux-musleabi/release/scan-uio-buffers ucla-rb102: 
#CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin liftof-rb --target=armv7-unknown-linux-musleabi --release && scp ../target/armv7-unknown-linux-musleabi/release/liftof-rb ucla-rb101: 
#CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin debug-idle --target=armv7-unknown-linux-musleabi --release && scp ../target/armv7-unknown-linux-musleabi/release/debug-idle ucla-rb51: 
#CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin watch-buffer-fill --target=armv7-unknown-linux-musleabi --release && scp ../target/armv7-unknown-linux-musleabi/release/watch-buffer-fill ucla-rb101: 
#scp ../target/armv7-unknown-linux-musleabi/release/liftof-rb tof-rb52:
