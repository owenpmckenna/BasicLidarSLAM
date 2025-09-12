extern crate serialport;

use std::thread::{sleep, Thread};
use std::time::Duration;
use rplidar_drv::ScanOptions;
//use rplidar_drv::{ScanMode, ScanOptions};
//use serial2::SerialPort;

fn main() {
    println!("Hello, world!");
    use rplidar_drv::RplidarDevice;
    //let serial_port = SerialPort::open("/dev/ttyUSB0".to_owned(), 115200).unwrap();
    let serial_port = serialport::new("/dev/ttyUSB0", 115200).open().unwrap();
    let mut rplidar = RplidarDevice::with_stream(serial_port);

    let device_info = rplidar.get_device_info().unwrap();
    println!("device info: {:?}", device_info);
    //let scan_type = rplidar.start_scan_with_options(&ScanOptions::force_scan()).unwrap();
    //rplidar.start_motor().expect("Motor start failed somehow");
    //println!("scan type: {:?}", scan_type);
    //let health = rplidar.get_device_health().unwrap();
    //println!("health: {:?}", health);

    sleep(Duration::from_secs(5));
    loop {
        for scan_point in rplidar.grab_scan_with_timeout(Duration::from_secs(15)).unwrap() {
            println!("dist: {}", scan_point.dist_mm_q2);
            println!("angle: {}", scan_point.angle_z_q14);
        }
    }
}
