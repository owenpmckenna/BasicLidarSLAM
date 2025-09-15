extern crate serialport;

use std::thread::{sleep, Thread};
use std::time::Duration;
use rplidar_drv::ScanOptions;
use serialport::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
//use rplidar_drv::{ScanMode, ScanOptions};
//use serial2::SerialPort;

fn main() {
    println!("Hello, world!");
    use rplidar_drv::RplidarDevice;
    //let serial_port = SerialPort::open("/dev/ttyUSB0".to_owned(), 115200).unwrap();\
    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(10),
    };
    let mut serial_port = serialport::open_with_settings("/dev/ttyUSB0", &s).unwrap();
    serial_port
        .write_data_terminal_ready(false)
        .expect("failed to clear DTR");
    let mut rplidar = RplidarDevice::with_stream(serial_port);

    let device_info = rplidar.get_device_info().unwrap();
    println!("device info: {:?}", device_info);
    rplidar.set_motor_pwm(500).expect("Motor start failed somehow");
    println!("start motor done");
    //sleep(Duration::from_secs(5));
    let scan_type = rplidar.start_scan_with_options(&ScanOptions::force_scan()).unwrap();
    println!("scan type: {:?}", scan_type);
    //let health = rplidar.get_device_health().unwrap();
    //println!("health: {:?}", health);

    sleep(Duration::from_secs(5));
    let mut x = 0;
    let mut y = 0;
    let mut total: f32 = 0.0;
    loop {
        for scan_point in rplidar.grab_scan_with_timeout(Duration::from_secs(15)).unwrap() {
            //println!("dist: {}", scan_point.distance());
            //println!("angle: {}", scan_point.angle());
            //println!("x: {}", x);
            total += scan_point.distance();
            x += 1;
        }
        println!("total: {}", total/(x as f32));
        println!("y: {}", y);
        y += 1;
        //x = 0;
    }
    //println!("Grab one point! {:?}", rplidar.grab_scan_point().unwrap())
}
//Scan types: Standard, Express, Boost, Sensitivity, Stability