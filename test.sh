#!/bin/bash

mkdir -p mountdir
cargo build && clear && sudo ./target/debug/bowl-rs --debug true -u 0 -m ./mountdir/ -c "/bin/bash" -a /lib64:/lib64 -a /lib:/lib
