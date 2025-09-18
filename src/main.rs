extern crate serialport;

use std::cmp::max;
use std::error::Error;
use std::process::exit;
use std::thread::{sleep, Thread};
use std::time::Duration;
use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::style::{Color, GREEN, WHITE};
use rplidar_drv::{Channel, RplidarHostProtocol, RposError, ScanOptions};
use serialport::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
//use rplidar_drv::{ScanMode, ScanOptions};
//use serial2::SerialPort;
fn polar_to_cartesian_radians(radius: f32, theta_radians: f32) -> (f32, f32) {
    let x = radius * theta_radians.cos();
    let y = radius * theta_radians.sin();
    (x, y)
}
fn grab_points() -> Result<Vec<(f32, f32)>, Box<dyn std::error::Error>> {
    use rplidar_drv::RplidarDevice;
    //let serial_port = SerialPort::open("/dev/ttyUSB0".to_owned(), 115200).unwrap();\
    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(100),
    };
    let mut serial_port = serialport::open_with_settings("/dev/ttyUSB0", &s)?;
    serial_port
        .write_data_terminal_ready(false)
        .expect("failed to clear DTR");
    let channel = Channel::<RplidarHostProtocol, dyn serialport::SerialPort>::new(
        RplidarHostProtocol::new(),
        serial_port,
    );
    let mut rplidar = RplidarDevice::new(channel);
    let mut x = 0;
    let mut y = 0;
    let mut total: f32 = 0.0;
    let mut data: Vec<(f32, f32)> = Vec::with_capacity(50000);
    rplidar.set_motor_pwm(500).expect("Motor start failed somehow");
    rplidar.start_motor().expect("Start motor failed");
    let scan_type = rplidar.start_scan_with_options(&ScanOptions::force_scan())?;
    'outer: for i in 0..50 {
        let scan_data_o = rplidar.grab_scan_with_timeout(Duration::from_secs(15));
        match scan_data_o {
            Ok(it) => {
                for scan_point in it {
                    //print!("{},", scan_point.distance());
                    //println!("{}", scan_point.angle());
                    let p = polar_to_cartesian_radians(scan_point.distance(), scan_point.angle());
                    data.push(p);
                    //println!("x: {}", x);
                    x += 1;
                }
            }
            Err(err) => {
                if let Some(RposError::OperationTimeout) = err.downcast_ref::<RposError>() {
                    println!("timeout...");
                    //continue;
                } else {
                    println!("Error: {:?}", err);
                    println!("Failed at: {}", i);
                    break 'outer;
                }
            }
        };
        //println!("total: {}", total/(x as f32));
        //println!("y: {}", y);
        y += 1;
        //x = 0;
    }
    rplidar.stop().expect("Stopping failed.");
    Ok(data)
}
fn main() {
    let root = BitMapBackend::new("../data.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).expect("Fill failed");
    //println!("Hello, world!");

    //rplidar.stop_motor().expect("Motor stop failed somehow");
    //rplidar.stop().expect("Stop failed somehow");
    //let device_info = rplidar.get_device_info().unwrap();
    //println!("device info: {:?}", device_info);
    //println!("start motor done");
    //sleep(Duration::from_secs(5));
    //println!("scan type: {:?}", scan_type);
    //let health = rplidar.get_device_health().unwrap();
    //println!("health: {:?}", health);

    //sleep(Duration::from_secs(5));
    let mut data: Vec<(f32, f32)> = Vec::with_capacity(50000);

    while data.len() < 15000 {
        match grab_points() {
            Ok(mut it) => {data.append(&mut it);}
            Err(it) => {println!("Error: {}", it); sleep(Duration::from_secs(5))}
        }
        println!("ran, now have {} points", data.len())
    }
    
    println!("number of points: {}", data.len());
    let mut mx: f32 = 0.0;
    let mut my: f32 = 0.0;
    for (x,y) in &data {
        mx = mx.max(x.abs());
        my = my.max(y.abs());
    }
    //let (xmin, xmax) = data.iter().map(|(x, _)| *x).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| (min.min(v), max.max(v)));
    //let (ymin, ymax) = data.iter().map(|(_, y)| *y).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| (min.min(v), max.max(v)));
    let (xmin, xmax) = (-mx, mx);
    let (ymin, ymax) = (-my, my);
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