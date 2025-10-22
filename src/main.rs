mod Lidar;

extern crate serialport;

use std::cmp::max;
use std::error::Error;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::exit;
use std::thread;
use std::thread::{sleep, Thread};
use std::time::Duration;
use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::coord::Shift;
use plotters::drawing::{DrawingArea, IntoDrawingArea};
use plotters::element::Circle;
use plotters::style::{Color, GREEN, WHITE};
use roboclaw::Roboclaw;
use rplidar_drv::{Channel, RplidarHostProtocol, RposError, ScanOptions};
use rppal::gpio::Gpio;
use serialport::{DataBits, FlowControl, Parity, StopBits};
use crate::Lidar::LidarUnit;

//use rplidar_drv::{ScanMode, ScanOptions};
//use serial2::SerialPort;
fn polar_to_cartesian_radians(radius: f32, theta_radians: f32) -> (f32, f32) {
    let x = radius * theta_radians.cos();
    let y = radius * theta_radians.sin();
    (x, y)
}
fn main() {
    /*let s = Settings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(100),
    };*/
    let serial_port = serialport::new("/dev/ttyACM0", 115200)
        .stop_bits(StopBits::One)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .timeout(Duration::from_millis(100))
        .open().unwrap();
    let mut rc = Roboclaw::new(serial_port);
    rc.forward_m1(16).expect("TODO: panic message");
    sleep(Duration::from_secs(5));
    rc.forward_m1(0).expect("TODO: panic message0");
    sleep(Duration::from_secs(5));
    exit(0);
    let root = BitMapBackend::new("../data.png", (1024, 768)).into_drawing_area();
    //println!("Hello, world!");
    let mut ld = LidarUnit::new();

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

    loop {
        println!("reading points...");
        ld.read_points().unwrap();
        present(&root, ld.get_data());
    }

    //println!("number of points: {}", data.len());

    //println!("Grab one point! {:?}", rplidar.grab_scan_point().unwrap())
    //present(&root, &data);
}
fn present(root: &DrawingArea<BitMapBackend, Shift>, data: &[(f32, f32); Lidar::DATA_LEN]) {
    root.fill(&WHITE).expect("Fill failed");
    let mut mx: f32 = 0.0;
    let mut my: f32 = 0.0;
    for (x,y) in data {
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
}
//Scan types: Standard, Express, Boost, Sensitivity, Stability