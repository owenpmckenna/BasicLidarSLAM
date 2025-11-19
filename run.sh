#!/bin/bash
sudo chmod +777 /dev/ttyUSB0
sudo chmod +777 /dev/ttyACM*
git pull
RUSTFLAGS="-C target-cpu=native" cargo run --release