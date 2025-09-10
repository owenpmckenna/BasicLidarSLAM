extern crate serialport;

use serial2::SerialPort;

fn main() {
    println!("Hello, world!");
    use rplidar_drv::RplidarDevice;
    //let serial_port = SerialPort::open("/dev/ttyUSB0".to_owned(), 115200).unwrap();
    let mut serial_port = serialport::new("/dev/ttyUSB0", 115200).open().unwrap();
    let mut rplidar = RplidarDevice::with_stream(serial_port);

    let device_info = rplidar.get_device_info().unwrap();
    rplidar.start_scan().unwrap();

    while true {
        let scan_point = rplidar.grab_scan_point().unwrap();

        println!("dist: {}", scan_point.dist_mm_q2);
        println!("angle: {}", scan_point.angle_z_q14);
    }
}
