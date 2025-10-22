TOFCPU information 
==================


How to set up natting? 
----------------------

Nat (Network address translation) allows a computer to access 
the internet through another machine (gateway) which has to 
pass on the traffic. This is usually necessary when we go 
from one network (e.g. 10.0.1.1/24 to 192.168.1.1/24). 
So the gateway machine would typically have 2 network interfaces.

* Step 1: Set the default gateway on the TOF computer 

  - Delete the old gateway `sudo ip route del default` 
  - Set the new gateway `sudo ip route add default via <GATEWAY IP>`

* Step 2: There are 4 steps which need to be taken to set the 
  gateway macnine to forward ip traffic. 
  This is taken from `this article <https://www.revsys.com/writings/quicktips/nat.html>`_. It will require super user rights. 
  
  - Make sure ip forwarding is allowed `echo 1 > /proc/sys/net/ipv4/ip_forward`
  - Set up the actual natting, allowing packets to go back and forth 
    `/sbin/iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
` 
    `/sbin/iptables -A FORWARD -i eth0 -o eth1 -m state --state RELATED,ESTABLISHED -j ACCEPT`
    `/sbin/iptables -A FORWARD -i eth1 -o eth0 -j ACCEPT` 

  where `eth0` is the external and `eth1` the internal network adapter. 

