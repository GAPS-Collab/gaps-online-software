#!/bin/bash
# it was "#! /bin/sh" before

get_version() {
  # build it for this architecture so we can parse the version
  cargo build --all-features --release --bin=liftof-cc
  VERSION=`../target/release/liftof-cc -V`
  IFS=' '
  read -ra arr <<< "$VERSION"
  for val in "${arr[@]}";
    do
      VERSION=$val;
    done
  echo "$VERSION"
}

#x86_64-unknown-linux-gnu
compile_and_deploy_target() {
  # the cross FAQ says to run cargo clean 
  # every time you switch targets.
  cargo clean
  #CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cargo build --bin $1 --features=tofcontrol --release && scp ../target/release/$1 $2:~/bin/ 
  
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --target=x86_64-unknown-linux-musl --bin $1 --release
  #scp ../target/x86_64-unknown-linux-musl/release/$1 $2:~/bin/liftof-cc-0.9.3-paolo
  scp ../target/x86_64-unknown-linux-musl/release/$1 $3:~/bin/liftof-cc-0.9.7-paolo
  #scp liftof-cc-config-0.9.3-paolo.toml $2:~/config/
  scp liftof-cc-config-0.9.7-paolo.toml $3:~/config/
  cargo clean
}

# first delete everything, since there might be remains of a previously issued cargo check
rm -rf ../target/x86_64-unknown-linux-musl/*

compile_and_deploy_target liftof-cc nevis-tof tof-computer

