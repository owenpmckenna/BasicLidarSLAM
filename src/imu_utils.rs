use lsm6dsox::{AccelerometerScale, DataRate, Lsm6dsox, SlaveAddress};
use lsm6dsox::accelerometer::Accelerometer;
use rppal::i2c::I2c;

pub fn get_imu_device() -> Lsm6dsox<I2c, i32> {
    let i2c = I2c::with_bus(1).expect("could not get i2c from rppal");
    let mut lsm = Lsm6dsox::new(i2c, SlaveAddress::Low, 1000);

    lsm.setup().expect("i2c setup fail");
    lsm.set_accel_sample_rate(DataRate::Freq52Hz).expect("i2c setup freq fail");
    lsm.set_accel_scale(AccelerometerScale::Accel16g).expect("i2c setup accel fail");
    if let Ok(reading) = lsm.accel_norm() {
        println!("Good first read acceleration: {:?}", reading);
    }
    lsm
}