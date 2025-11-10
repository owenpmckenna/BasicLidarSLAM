mod Lidar;
mod Drivetrain;
mod Webserver;
mod LidarLocalizer;

extern crate serialport;

use std::cmp::max;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::exit;
use std::thread;
use std::thread::{sleep, Thread};
use std::time::Duration;
use crossbeam_channel::unbounded;
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
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use tokio::runtime::Runtime;
use crate::Lidar::LidarUnit;
use crate::LidarLocalizer::{InstantLidarLocalizer, Line};
use crate::Webserver::{SendData, SmallData};

//use rplidar_drv::{ScanMode, ScanOptions};
//use serial2::SerialPort;
fn polar_to_cartesian_radians(radius: f32, theta_radians: f32) -> (f32, f32) {
    let x = radius * theta_radians.cos();
    let y = radius * theta_radians.sin();
    (x, y)
}
fn main() {
    env_logger::init();
    let mut ld = LidarUnit::new();
    let (tx, rx) = unbounded::<SendData>();
    let rt = Runtime::new().unwrap();
    rt.spawn(async {
        let webserver = Webserver::Webserver::new(rx).await;
        webserver.serve().await;
    });
    let t = thread::spawn(move || {
        let mut localizer = LidarLocalizer::LidarLocalizer::new();
        loop {
            let points: Vec<(f32, f32)> = ld.grab_points().unwrap().iter()
                // / 6.0 * 400.0 
                .map(|it| { polar_to_cartesian_radians(it.0, it.1) })
                .collect();
            let data = points.iter().map(|it| { SmallData { x: (it.0 / 6.0 * 400.0) as i32, y: (it.1 / 6.0 * 400.0) as i32 } }).collect();
            let i_localizer = InstantLidarLocalizer::new((0.0,0.0), 1000.0, &points);
            localizer.process(i_localizer);
            //println!("got {} points!", points.len());
            let to_send = SendData {data, lines: localizer.clone_lines(|x| x / 6.0 * 400.0)};
            tx.send(to_send).unwrap();
            sleep(Duration::from_millis(50));
        }
    });
    t.join().expect("");
    /*let mut dt = Drivetrain::Drivetrain::new();
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(
        stdout,
        "q to exit."
    )
        .unwrap();
    stdout.flush().unwrap();
    /*let s = Settings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(100),
    };*/
    
    let mut loopnum = 0;
    for k in stdin.keys() {
        println!("running... (loop {})", loopnum);
        loopnum += 1;
        write!(
            stdout,
            "{}",
            termion::clear::CurrentLine
        ).unwrap();

        match k.unwrap() {
            Key::Char('q') => break,
            Key::Alt(c) => println!("^{}", c),
            Key::Ctrl(c) => println!("*{}", c),
            Key::Esc => break,
            Key::Char('w') => { dt.x += 0.1; },
            Key::Char('a') => { dt.y -= 0.1; },
            Key::Char('s') => { dt.x -= 0.1; },
            Key::Char('d') => { dt.y += 0.1; },
            Key::Left => { dt.turn -= 0.1; },
            Key::Right => { dt.turn += 0.1; },
            Key::Backspace => { dt.x = 0.0; dt.y = 0.0; dt.turn = 0.0; },
            x => {
                println!("{:?}", x)
            }
        }
        dt.power().expect("something failed idk");
        stdout.flush().unwrap();
    }
    dt.x = 0.0; dt.y = 0.0; dt.turn = 0.0;
    dt.power().expect("failed");
    write!(stdout, "{}", termion::cursor::Show).unwrap();
    stdout.suspend_raw_mode().expect("could not suspend raw mode");
    exit(0);*/
    //let root = BitMapBackend::new("../data.png", (1024, 768)).into_drawing_area();
    //println!("Hello, world!");
    //let mut ld = LidarUnit::new();

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

    /*loop {
        println!("reading points...");
        ld.read_points().unwrap();
        present(&root, ld.get_data());
    }*/

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