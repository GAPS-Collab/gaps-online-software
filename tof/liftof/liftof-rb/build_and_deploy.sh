#! /bin/sh

compile_and_deploy_target() {
  CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin $1 --target=armv7-unknown-linux-musleabi --release && scp ../target/armv7-unknown-linux-musleabi/release/$1 $2:~/bin/ 
}

# first delete everything, since there might be remains of a previously issued cargo check
rm -rf ../target/armv7-unknown*

#compile_and_deploy_target liftof-rb ucla-rb149
#for rb in ucla-rb00 ucla-rb01 ucla-rb02 ucla-rb03 ucla-rb04 ucla-rb07 ucla-rb08 ucla-rb09 ucla-rb10;
for rb in tof-rb05 tof-rb06
do compile_and_deploy_target liftof-rb $rb;
done

#compile_and_deploy_target debug-idle
#compile_and_deploy_target watch-buffer-fill

