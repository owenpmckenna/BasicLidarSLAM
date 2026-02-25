use std::error::Error;
use std::time::Duration;
use rplidar_drv::{Channel, RplidarDevice, RplidarHostProtocol, RposError, ScanOptions};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

pub(crate) const DATA_LEN: usize = 10000;
const TIMEOUTS_MAX: usize = 10;
const FATALS_MAX: usize = 100;
fn polar_to_cartesian_radians(radius: f32, theta_radians: f32) -> (f32, f32) {
    let x = radius * theta_radians.cos();
    let y = radius * theta_radians.sin();
    (x, y)
}
pub struct LidarUnit {
    lidar_dev: Option<RplidarDevice<dyn SerialPort>>,
    data: [(f32, f32); DATA_LEN],
    data_index: usize,
    fatals: usize,
    timeouts: usize
}
impl LidarUnit {
    pub(crate) fn get_data(&self) -> &[(f32, f32); DATA_LEN] {
        &self.data
    }
    fn get_rplidar() -> Result<RplidarDevice<dyn SerialPort>, Box<dyn Error>> {
        let mut serial_port = serialport::new("/dev/ttyUSB0", 115200)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .timeout(Duration::from_millis(1000))
            .open().unwrap();
        //let mut serial_port = serialport::new("/dev/ttyUSB0", &s)?;
        serial_port
            .write_data_terminal_ready(false)
            .expect("failed to clear DTR");
        let channel = Channel::<RplidarHostProtocol, dyn SerialPort>::new(
            RplidarHostProtocol::new(),
            serial_port,
        );
        let mut dev = RplidarDevice::new(channel);
        let mut scan_modes = dev.get_all_supported_scan_modes().expect("could not get scan modes");
        scan_modes.sort_by_key(|it| (it.us_per_sample * 1000.0) as u128);
        for i in &scan_modes {
            println!("scan mode: id:{}, name:{}, us per sample: {}, max dist: {}", i.id, i.name, i.us_per_sample, i.max_distance)
        }
        //we want the lowest us per sample
        dev.start_scan_with_options(&ScanOptions::force_scan_with_mode(scan_modes.first().unwrap().id))?;
        let _ = dev.grab_scan_point();//Ignore result
        Ok(dev)
    }
    pub(crate) fn new() -> LidarUnit {
        let rplidar = match Self::get_rplidar() {
            Ok(it) => {Some(it)}
            Err(err) => {println!("Error: {}", err); None}
        };
        LidarUnit {lidar_dev: rplidar, data: [(0f32, 0f32); DATA_LEN], data_index: 0, fatals: 0, timeouts: 0 }
    }
    fn regen_connection(&mut self) -> Option<()> {
        {
            //attempt to force disconnect
            self.lidar_dev = None
        }
        self.lidar_dev = Some(match Self::get_rplidar() {
            Ok(it) => {it}
            Err(err) => {
                println!("Error (regen connection): {}", err);
                self.fatals += 1;
                if self.fatals > FATALS_MAX {
                    return None
                }
                return self.regen_connection()
            }
        });
        Some(())
    }
    fn grab_single_point(&mut self) -> Result<(f32, f32), ()> {
        match self.lidar_dev.as_mut().unwrap().grab_scan_point() {
            Ok(it) => {Ok(polar_to_cartesian_radians(it.distance(), it.angle()))}
            Err(it) => {
                if let Some(RposError::OperationTimeout) = it.downcast_ref::<RposError>() {
                    self.timeouts += 1;
                    if self.timeouts > TIMEOUTS_MAX {
                        self.fatals += 1;
                        self.timeouts = 0;
                        match self.regen_connection() {
                            None => {return Err(())} Some(_) => {}
                        };
                    }
                    self.grab_single_point()
                } else {
                    self.fatals += 1;
                    if self.fatals > FATALS_MAX {
                        Err(())
                    } else {
                        match self.regen_connection() {
                            None => {return Err(())} Some(_) => {}
                        };
                        self.grab_single_point()
                    }
                }
            }
        }
    }
    pub(crate) fn read_single_point(&mut self) -> Result<(), ()> {
        let num = self.grab_single_point()?;
        self.data[self.data_index % self.data.len()] = num;
        self.data_index += 1;
        Ok(())
    }
    pub fn grab_points(&mut self) -> Result<Vec<(f32, f32)>, ()> {
        match self.lidar_dev.as_mut().unwrap().grab_scan_with_timeout(Duration::from_secs(15)) {
            Ok(it) => {
                Ok(it.iter().filter(|it| {/*println!("ang:{}, dist:{}", it.angle(), it.distance());*/ it.is_valid()}).map(|it| {(it.distance(), it.angle())}).collect())
            }
            Err(it) => {
                if let Some(RposError::OperationTimeout) = it.downcast_ref::<RposError>() {
                    println!("timeout...");
                    self.timeouts += 1;
                    if self.timeouts > TIMEOUTS_MAX {
                        println!("Timeouts exceeded normal value, regenerating...");
                        self.fatals += 1;
                        self.timeouts = 0;
                        match self.regen_connection() {
                            None => {return Err(())}
                            Some(_) => {}
                        };
                    }
                    self.grab_points()
                } else {
                    println!("Error: {:?}, fatals: {}", it, self.fatals);
                    self.fatals += 1;
                    if self.fatals > FATALS_MAX {
                        Err(())
                    } else {
                        match self.regen_connection() {
                            None => {return Err(())}
                            Some(_) => {}
                        };
                        self.grab_points()
                    }
                }
            }
        }
    }
    pub(crate) fn read_points(&mut self) -> Result<(), ()> {
        let points = self.grab_points()?;
        //println!("read {} points", points.len());
        for i in points {
            self.data[self.data_index % self.data.len()] = i;
            self.data_index += 1;
        }
        Ok(())
    }
}