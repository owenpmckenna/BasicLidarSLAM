extern crate serialport;

use std::thread::{sleep, Thread};
use std::time::Duration;
use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::style::{Color, GREEN, WHITE};
use rplidar_drv::ScanOptions;
use serialport::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
//use rplidar_drv::{ScanMode, ScanOptions};
//use serial2::SerialPort;
fn polar_to_cartesian_radians(radius: f32, theta_radians: f32) -> (f32, f32) {
    let x = radius * theta_radians.cos();
    let y = radius * theta_radians.sin();
    (x, y)
}
fn main() {
    let root = BitMapBackend::new("../data.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).expect("Fill failed");
    //println!("Hello, world!");
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
    //println!("device info: {:?}", device_info);
    rplidar.set_motor_pwm(500).expect("Motor start failed somehow");
    //println!("start motor done");
    //sleep(Duration::from_secs(5));
    //println!("scan type: {:?}", scan_type);
    //let health = rplidar.get_device_health().unwrap();
    //println!("health: {:?}", health);

    sleep(Duration::from_secs(5));
    let mut x = 0;
    let mut y = 0;
    let mut total: f32 = 0.0;
    let mut data: Vec<(f32, f32)> = Vec::new();
    rplidar.set_motor_pwm(500).expect("Motor start failed somehow");
    let scan_type = rplidar.start_scan_with_options(&ScanOptions::force_scan()).unwrap();
    for i in 0..1000 {
        let scan_point = rplidar.grab_scan_point_with_timeout(Duration::from_secs(15)).unwrap();
        print!("{},", scan_point.distance());
        println!("{}", scan_point.angle());
        let p = polar_to_cartesian_radians(scan_point.distance(), scan_point.angle());
        data.push(p);
        //println!("x: {}", x);
        x += 1;
        //println!("total: {}", total/(x as f32));
        //println!("y: {}", y);
        y += 1;
        //x = 0;
    }
    rplidar.stop().expect("Stopping failed.");
    let (xmin, xmax) = data.iter().map(|(x, _)| *x).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| (min.min(v), max.max(v)));
    let (ymin, ymax) = data.iter().map(|(_, y)| *y).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| (min.min(v), max.max(v)));
    let mut scatter_ctx = ChartBuilder::on(&root)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(xmin..xmax, ymin..ymax)
        .unwrap();
    scatter_ctx
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .unwrap();
    scatter_ctx.draw_series(
        data
            .iter()
            .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
    ).unwrap();
    root.present().expect("Unable to write result to file");
    //println!("Grab one point! {:?}", rplidar.grab_scan_point().unwrap())
}
//Scan types: Standard, Express, Boost, Sensitivity, Stability