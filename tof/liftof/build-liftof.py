#! /usr/bin/env python

"""
Build and deploy different liftof components
"""
import subprocess as sub
import os
import shutil
import tqdm

def all_rbs():
    """
    Get all avaiable readoutboards
    """
    import gaps_online.db as db
    rbs = db.ReadoutBoard.objects.all()
    rb_ids = sorted([k.rb_id for k in rbs])
    #print (rb_ids)
    #raise
    return rb_ids

def get_version(binary):
    """
    Get the version from the -V argument
    """
    if binary == 'liftof-scheduler':
        os.chdir('liftof-cc')
    else:
        os.chdir(f'{binary}')
    print('=> Running build command..')
    build_cmd = f"cargo build -j 24 --all-features --bin={binary}"
    result = sub.run([build_cmd], shell=True)
    print(' .. complete!')
    version_cmd = f"../target/debug/{binary} -V | tail -n 1"
    result = sub.run([version_cmd], shell=True, capture_output=True, text=True)
    version = result.stdout.split()[1]
    print (f'=> Using {binary} version: {version}')
    os.chdir('..')
    return version

def build_for_muslx86_64(binary, njobs=24, clean=False):
    if binary == 'liftof-scheduler':
        os.chdir('liftof-cc')
    else:
        os.chdir(f'{binary}')
    if clean:
        sub.run(["cargo clean"], shell=True)
    build_cmd = f'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build -j {njobs} --target=x86_64-unknown-linux-musl --bin {binary} --release --all-features'
    result = sub.run([build_cmd], shell=True)
    shutil.move(f'../target/x86_64-unknown-linux-musl/release/{binary}', '../build/')
    os.chdir('..')
    version_cmd = f"build/{binary} -V | tail -n 1"
    result = sub.run([version_cmd], shell=True, capture_output=True, text=True)
    version = result.stdout.split()[1]
    shutil.copy(f'build/{binary}', f'releases/{binary}.musl.x86.{version}')

def build_for_arm32(binary, njobs=24, clean=False):
    """
    Readoutboards have ARM32 architecture
    """
    os.chdir(f'{binary}')
    if clean:
        sub.run(["cargo clean"], shell=True)
    build_cmd = f'CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build -j {njobs} --bin {binary} --target=armv7-unknown-linux-musleabi --all-features --release' 
    result = sub.run([build_cmd], shell=True)
    shutil.move(f'../target/armv7-unknown-linux-musleabi/release/{binary}', '../build/')
    os.chdir('..')

def pack(binary):
    """
    Pack the binary together with the auxiliary 
    files it needs in a .tar.gz
    """
    # for liftof-cc this means
    # liftof-cc, gaps_flight.db, liftof.service
    # new liftof-config file
    pass


def deploy(binary, dest_dir='bin'):
    #version = get_version(binary)
    version = '0.10.3'
    if binary == 'liftof-cc':
       deploy_cmd = f"rsync -avz build/{binary} tofcpu-pl:{dest_dir}/{binary}.{version}"
       sub.run([deploy_cmd], shell=True)
		   
    elif binary == 'liftof-rb':
        rbids = all_rbs() 
        rb_targets = [f'tof-rb{k:02}' for k in rbids]
        for target in tqdm.tqdm(rb_targets, desc='Deploying to rbs'):
            deploy_cmd = f"rsync -avz build/{binary} tofcpu-pl:{dest_dir}/{binary}.{version}"
            sub.run([deploy_cmd], shell=True)
     
    elif binary == 'liftof-tui':
        deploy_cmd = f"rsync -avz build/{binary} gaps-gse5:tof-moni/{binary}.{version}"
        deploy_cmd1 = f"rsync -avz build/{binary} tof-gse5:tof-moni/{binary}.{version}"
        sub.run([deploy_cmd], shell=True)
        sub.run([deploy_cmd1], shell=True)
    
def deploy_asset(asset):
    if asset == 'db':
        rbids = all_rbs() 
        rb_targets = [f'tof-rb{k:02}' for k in rbids]
        for target in ['tofcpu-pl']:
            deploy_cmd = f"rsync -avz ../../gaps-db/gaps_db/gaps_flight.db {target}:config/gaps_flight.db"
            sub.run([deploy_cmd], shell=True)
        for target in tqdm.tqdm(rb_targets, desc='Deploying to rbs'):
            deploy_cmd = f"rsync -avz ../../gaps-db/gaps_db/gaps_flight.db {target}:config/gaps_flight.db"
            sub.run([deploy_cmd], shell=True)


    pass

if __name__ == '__main__':
    
    import argparse
    import sys

    parser = argparse.ArgumentParser(description="Build and deploy various liftof components!")
    subparsers =  parser.add_subparsers(help='Available commands', dest='cmd')
    buildparser = subparsers.add_parser('build', help='Build liftof components')
    buildparser.add_argument("-j", type=int, default=24, help="Use <j> number of cores")
    buildparser.add_argument("--no-musl", action="store_true", help="Do not use musl as libc replacement (not recommended)")
    buildparser.add_argument("--get-version", action="store_true", help="Get the lastest version string from the compiled binary!")
    buildparser.add_argument("--clean", action="store_true", help="Run a 'cargo clean' before the build")
    buildparser.add_argument("binary", type=str, help="Select the binary to build")

    deployparser = subparsers.add_parser('deploy', help='Deploy liftof components')
    deployparser.add_argument("-j", type=int, default=24, help="Use <j> number of cores")
    deployparser.add_argument("--no-musl", action="store_true", help="Do not use musl as libc replacement (not recommended)")
    deployparser.add_argument("--debug", action="store_true", help="Deploy to debug directory", default=False)
    deployparser.add_argument("--tofcpu-ssh-name", type=str, help="The name of the tof-cpu in .ssh/config", default="tofcpu-pl")
    deployparser.add_argument("binary", type=str, help="Select the binary to build")
    deployassparser = subparsers.add_parser('deploy_asset', help='Deploy liftof components')
    deployassparser.add_argument("asset", type=str, help="Asset to deply. Choices are 'db'")
    args = parser.parse_args()
    if len(vars(args).keys()) == 0:
        parser.print_help()
        parser.exit()
    #if args.get_version:
    #    get_version(args.binary)
    #    sys.exit(0)
    if args.cmd == 'build' or args.cmd == 'deploy':
        try:
            shutil.rmtree('build')
        except Exception as e:
            print ('not deleting build directory...{e}')
        os.makedirs('build', exist_ok=True)
        os.makedirs('releases', exist_ok=True)
        if args.binary == 'liftof-rb':
            build_for_arm32(args.binary, njobs=args.j,clean=args.clean)
#bui    ld_for_muslx86_64('liftof-cc')
        else:
            #if args.no_musl:
            #    build_for_gnux86_64(arggs.binary, njobs=args.j)
            #else:
            build_for_muslx86_64(args.binary, njobs=args.j, clean=args.clean)
        if args.cmd == 'deploy':
            if args.debug:
                deploy(args.binary, dest_dir='bin/debug')
            else:
                deploy(args.binary)
    print (args.cmd)
    if args.cmd == 'deploy_asset':
        deploy_asset(args.asset)
#deploy()
#print (get_version('liftof-cc'))
