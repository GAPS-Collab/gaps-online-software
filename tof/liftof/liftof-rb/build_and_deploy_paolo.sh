#! /bin/sh
get_version() {
  # build it for this architecture so we can parse the version
  cargo build --all-features --release --bin=liftof-rb
  VERSION=`../target/release/liftof-rb -V`
  IFS=' '
  read -ra arr <<< "$VERSION"
  for val in "${arr[@]}";
    do
      VERSION=$val;
    done
  echo "$VERSION"
}

deploy_target() {
  scp ../target/armv7-unknown-linux-musleabi/release/$1 $3:~/bootstrap-tof/$1-$2
}

compile_target() {
  # first delete everything, since there might be remains of a previously issued cargo check
  rm -rf ../target/armv7-unknown*
  #cross clean
  CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin $1 --target=armv7-unknown-linux-musleabi --all-features --release 
}

compile_and_deploy_target() {
  compile_target $1 $2
  cp ../target/armv7-unknown-linux-musleabi/release/$1 ../target/armv7-unknown-linux-musleabi/release/$1-$version
  deploy_target $1 $version $2
}

# UCLA test stand
UCLA_RB="ucla-tof-rb47 ucla-tof-rb33 ucla-tof-rb34"
#UCLA_RB="ucla-tof-rb05 ucla-tof-rb28 ucla-tof-rb33 ucla-tof-rb34"
UCLA_RB="nevis-tof"

compile_target liftof-rb
version=$(get_version)
for rb in `echo $UCLA_RB`;
  do
    echo "Deploying liftof-rb V$version to $rb" 
    deploy_target liftof-rb $version $rb;
    scp configs/liftof-rb-config-0.9.3.json nevis-tof:bootstrap-tof/
done;
#for rb in `echo $UCLA_RB`; 
#  do echo $rb;
#  scp liftof.service $rb:bin/;
#  scp -r configs $rb:config;
#done
