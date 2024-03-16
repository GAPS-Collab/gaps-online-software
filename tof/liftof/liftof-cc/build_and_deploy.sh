#!/bin/bash
# it was "#! /bin/sh" before

get_version() {
  # build it for this architecture so we can parse the version
  cargo build --all-features --release --bin=liftof-cc
  VERSION=`../target/release/liftof-cc -V | tail -n 1`
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
  version=`get_version`
  cargo clean
  echo -e "-- [compile_and_deploy] Deploying version $version"
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --target=x86_64-unknown-linux-musl --bin $1 --release && rsync -avz ../target/x86_64-unknown-linux-musl/release/$1 $2:~/bin/liftof-cc-$version 
  echo -e "-- [compile_and_deploy] Starting rsync..."
  scp ../resources/config/liftof-config-$version.toml $2:~/config/
  cargo clean
}

# first delete everything, since there might be remains of a previously issued cargo check
rm -rf ../target/x86_64-unknown-linux-musl/*

compile_and_deploy_target liftof-cc nevis-tof
