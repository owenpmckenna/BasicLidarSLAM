export PKG_CONFIG_ALLOW_CROSS=1
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
cross build --target aarch64-unknown-linux-gnu --release

scp ./target/aarch64-unknown-linux-gnu/release/BasicLidarSLAM baylorlab@$LIDAR_ADDR:/home/baylorlab/SLAM
#export LIDAR_ADDR=10.64.0.0