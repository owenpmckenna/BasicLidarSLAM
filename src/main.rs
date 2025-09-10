extern crate serialport;

use std::time::Duration;
use serial2::SerialPort;

fn main() {
    println!("Hello, world!");
    use rplidar_drv::RplidarDevice;
    //let serial_port = SerialPort::open("/dev/ttyUSB0".to_owned(), 115200).unwrap();
    let mut serial_port = serialport::new("/dev/ttyUSB0", 115200).open().unwrap();
    let mut rplidar = RplidarDevice::with_stream(serial_port);

    let device_info = rplidar.get_device_info().unwrap();
    println!("device info: {:?}", device_info);
    let scan_type = rplidar.start_scan().unwrap();
    println!("scan type: {:?}", scan_type);

    while true {
        let scan_point = rplidar.grab_scan_point_with_timeout(Duration::from_secs(5)).unwrap();

        println!("dist: {}", scan_point.dist_mm_q2);
        println!("angle: {}", scan_point.angle_z_q14);
    }
}
