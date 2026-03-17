extern crate core;
extern crate serialport;
mod Lidar;
mod Drivetrain;
mod Webserver;
mod LidarLocalizer;

use crate::Lidar::LidarUnit;
use crate::LidarLocalizer::InstantLidarLocalizer;
use crate::Webserver::{SendData, SmallData};
use crossbeam_channel::unbounded;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::runtime::Runtime;

fn polar_to_cartesian_radians(radius: f32, theta_radians: f32) -> (f32, f32) {
    let x = radius * theta_radians.cos();
    let y = radius * theta_radians.sin();
    (x, y)
}
fn cartesian_to_polar_radians(x: f32, y: f32) -> (f32, f32) {//no idea if this is right, fyi
    let radius = (x * x + y * y).sqrt();
    let theta = y.atan2(x); // returns angle in radians
    (radius, theta)
}
#[inline(always)]
fn cartesian_to_polar_radians_theta(x: f32, y: f32) -> f32 {//no idea if this is right, fyi
    let theta = y.atan2(x); // returns angle in radians
    theta
}
fn main() {
    //env_logger::init();
    let mut ld = LidarUnit::new().expect("could not get lidar unit");
    println!("created lidar unit!");
    let (tx_points, rx_points) = unbounded::<Vec<(f32, f32)>>();
    let (tx_lines, rx_lines) = unbounded::<(Vec<SmallData>, InstantLidarLocalizer)>();
    let rx_points_arc = Arc::new(rx_points);
    let tx_lines_arc = Arc::new(tx_lines);
    let (tx, rx) = unbounded::<SendData>();
    let rt = Runtime::new().unwrap();
    rt.spawn(async {
        let webserver = Webserver::Webserver::new(rx).await;
        webserver.serve().await;
    });
    let t = thread::spawn(move || {
        println!("running!");
        let mut last_grab_time = Instant::now();
        loop {
            let time = last_grab_time.elapsed();
            //println!("grabbing points... elapsed: {}", time.subsec_millis() as u64 + time.as_secs() * 1000);
            last_grab_time = Instant::now();
            let mut points0: Vec<(f32, f32)> = ld.grab_points().expect("could not grab points");
            points0.sort_by(|a, b| a.1.total_cmp(&b.1)); //this shouldn't be needed but this isn't python so we can afford it
            let points: Vec<(f32, f32)> = points0.iter()
                .map(|it| { polar_to_cartesian_radians(it.0, it.1) })
                .collect();
            tx_points.send(points).unwrap();
        }
    });
    for i in 0..3 {
        let rx_points_arc = rx_points_arc.clone();
        let tx_lines_arc = tx_lines_arc.clone();
        thread::spawn(move || {
            loop {
                let points = rx_points_arc.recv().unwrap();
                //println!("got {} points in t2", points.len());
                if points.len() < 20 {
                    continue;
                }
                let data = points.iter().map(|it| { SmallData { x: it.0, y: it.1 } }).collect::<Vec<SmallData>>();
                let i_localizer = InstantLidarLocalizer::new((0.0, 0.0), 1000.0, &points);
                tx_lines_arc.send((data, i_localizer)).unwrap();
            }
        });
    }
    let t2 = thread::spawn(move || {
        let mut localizer = LidarLocalizer::LidarLocalizer::new();
        loop {
            let (data, ill) = rx_lines.recv().unwrap();
            if rx_lines.len() > 0 {
                println!("warning: rx_lines backed up with {} lines", rx_lines.len());
            }
            let mut old_lines = localizer.process(ill);

            //println!("got {} points!", points.len());
            let to_send = SendData { data, lines: old_lines, full_lines: localizer.clone_lines(), x: localizer.pos.0, y: localizer.pos.1, heading: localizer.heading };
            tx.send(to_send).unwrap();
            //sleep(Duration::from_millis(50));
        }
    });
    t.join().expect("");
    t2.join().expect("");
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
//Scan types: Standard, Express, Boost, Sensitivity, Stability
pub fn add(m: (f32, f32), rhs: (f32, f32)) -> (f32, f32) {
    (m.0 + rhs.0, m.1 + rhs.1)
}
pub fn sub(m: (f32, f32), rhs: (f32, f32)) -> (f32, f32) {
    (m.0 - rhs.0, m.1 - rhs.1)
}
