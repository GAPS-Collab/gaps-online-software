#!/usr/bin/perl -w

use strict;

$| = 1;

while (my $l = <STDIN>) {
    my @v = $l =~ m/=([^ ]+) /g;
    next if @v < 3;

    #print $l;
    #print join("\n", @v);
    print "$v[0] $v[2]\n";
}
