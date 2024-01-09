#! /usr/bin/env python

with open("tof-ssh-config","w") as cfg:
    for rbid in range(1,51):
        info = f"""host tof-rb{rbid:02}
    User gaps
    Hostname 10.0.1.1{rbid:02}
    ProxyJump tof-computer
    SetEnv TERM=xterm-256color"""
        cfg.write(info)
        cfg.write("\n\n")


