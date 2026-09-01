use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, Sub};
use lsm6dsox::accelerometer::Accelerometer;
use lsm6dsox::{AccelerometerScale, DataRate, Lsm6dsox, SlaveAddress};
use rppal::hal::Delay;
use rppal::i2c::I2c;
use std::time::{Duration, Instant};
use lsm6dsox::accelerometer::vector::F32x3;

pub type IMU = Lsm6dsox<I2c, Delay>;
pub fn get_imu_device() -> IMU {
    let i2c = I2c::with_bus(1).expect("could not get i2c from rppal");
    let mut lsm = Lsm6dsox::new(i2c, SlaveAddress::Low, Delay::new());

    lsm.setup().expect("i2c setup fail");
    lsm.set_accel_sample_rate(DataRate::Freq6660Hz).expect("i2c setup freq fail");
    lsm.set_accel_scale(AccelerometerScale::Accel2g).expect("i2c setup accel fail");
    let _ = lsm.accel_norm();
    lsm
}
pub fn calculate_wrongness(imu: &mut IMU) -> F323 {
    let mut samples = 0.0;
    let mut failed_samples = 0;
    let mut total = F323::default();
    let time = Instant::now();
    while time.elapsed() < Duration::from_secs(5) {
        match imu.accel_norm().map(|it| F323::from(it)) {
            Ok(it) => {
                total += it;
                samples += 1.0;
            }
            Err(_) => {
                failed_samples += 1;
            }
        }
    }
    total /= samples;
    println!("calculated wrongness of {} with {} samples after 5 sec", total, samples);
    total
}
pub fn print_accel_data(imu: &mut IMU, bias: F323) -> Option<F323> {
    match imu.accel_norm() {
        Ok(it) => {
            let out = F323::from(it) - bias;
            //println!("acceleration read: x{:+.3}, y{:+.3} z{:+.3}", out.0, out.1, out.2);
            Some(out)
        }
        Err(it) => {
            println!("error reading accel: {:?}", it);
            None
        }
    }
}
#[derive(Clone, Copy)]
pub struct F323 {
    data: [f32; 3]
}
impl F323 {
    pub fn new(a: f32, b: f32, c: f32) -> F323 {
        Self {
            data: [a, b, c]
        }
    }
}
impl Default for F323 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}
impl From<(f32, f32, f32)> for F323 {
    fn from((a, b, c): (f32, f32, f32)) -> Self {
        F323::new(a, b, c)
    }
}
impl Into<(f32, f32, f32)> for F323 {
    fn into(self) -> (f32, f32, f32) {
        (self.data[0], self.data[1], self.data[2])
    }
}

impl From<F32x3> for F323 {
    fn from(value: F32x3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}
impl Into<F32x3> for F323 {
    fn into(self) -> F32x3 {
        F32x3::new(self.data[0], self.data[1], self.data[2])
    }
}
impl Index<usize> for F323 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl Add<f32> for F323 {
    type Output = F323;

    fn add(self, rhs: f32) -> Self::Output {
        F323::new(self.data[0] + rhs, self.data[1] + rhs, self.data[2] + rhs)
    }
}
impl Add<F323> for F323 {
    type Output = F323;

    fn add(self, rhs: F323) -> Self::Output {
        F323::new(self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2])
    }
}

impl AddAssign<F323> for F323 {
    fn add_assign(&mut self, rhs: F323) {
        *self = *self + rhs;
    }
}
impl Mul<f32> for F323 {
    type Output = F323;
    fn mul(self, rhs: f32) -> Self::Output {
        F323::new(self.data[0] * rhs, self.data[1] * rhs, self.data[2] * rhs)
    }
}
impl Sub<F323> for F323 {
    type Output = F323;

    fn sub(self, rhs: F323) -> Self::Output {
        Self::new(self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2])
    }
}
impl Div<f32> for F323 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self[0] / rhs, self[1] / rhs, self[2] / rhs)
    }
}
impl DivAssign<f32> for F323 {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}
impl Sum for F323 {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        let mut start = Self::default();
        iter.for_each(|it| {
            start += it;
        });
        start
    }
}
impl Display for F323 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self[0], self[1], self[2])
    }
}