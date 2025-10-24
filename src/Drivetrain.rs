use std::time::Duration;
use roboclaw::Roboclaw;
use serialport::{DataBits, FlowControl, Parity, StopBits};

pub struct Drivetrain {
    r1: Roboclaw,
    r2: Roboclaw,
    pub x: f32,
    pub y: f32,
    pub turn: f32,
    pub s: f32
}
impl Drivetrain {
    pub fn new() -> Drivetrain {
        let serial_port = serialport::new("/dev/ttyACM0", 115200)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .timeout(Duration::from_millis(100))
            .open().unwrap();
        let mut rc = Roboclaw::new(serial_port);
        let serial_port_2 = serialport::new("/dev/ttyACM1", 115200)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .timeout(Duration::from_millis(100))
            .open().unwrap();
        let mut rc_2 = Roboclaw::new(serial_port_2);
        Drivetrain {r1: rc, r2: rc_2, x: 0.0, y: 0.0, turn: 0.0, s: 1.0 }
    }

    pub fn power(&mut self) -> std::io::Result<()> {
        self.fl_power((self.y + self.x + self.turn) * self.s)?;
        self.bl_power((self.y - self.x + self.turn) * self.s)?;
        self.fr_power((self.y - self.x - self.turn) * self.s)?;
        self.br_power((self.y + self.x - self.turn) * self.s)
    }

    fn bl_power(&mut self, speed: f32) -> std::io::Result<()> {
        self.set_bl((speed * 255.0) as i16)
    }
    fn br_power(&mut self, speed: f32) -> std::io::Result<()> {
        self.set_br((speed * 255.0) as i16)
    }
    fn fl_power(&mut self, speed: f32) -> std::io::Result<()> {
        self.set_fl((speed * 255.0) as i16)
    }
    fn fr_power(&mut self, speed: f32) -> std::io::Result<()> {
        self.set_fr((speed * 255.0) as i16)
    }
    fn set_bl(&mut self, speed: i16) -> std::io::Result<()> {//ok actually a u8 but whatever. [-255, 255]
        if speed > 0 {
            self.r2.forward_m2(speed as u8)
        } else {
            self.r2.backward_m2((-speed) as u8)
        }
    }
    fn set_fl(&mut self, speed: i16) -> std::io::Result<()> {//ok actually a u8 but whatever. [-255, 255]
        if speed > 0 {
            self.r2.forward_m1(speed as u8)
        } else {
            self.r2.backward_m1((-speed) as u8)
        }
    }
    fn set_fr(&mut self, speed: i16) -> std::io::Result<()> {//ok actually a u8 but whatever. [-255, 255]
        if speed > 0 {
            self.r1.forward_m2(speed as u8)
        } else {
            self.r1.backward_m2((-speed) as u8)
        }
    }
    fn set_br(&mut self, speed: i16) -> std::io::Result<()> {//ok actually a u8 but whatever. [-255, 255]
        if speed > 0 {
            self.r1.forward_m1(speed as u8)
        } else {
            self.r1.backward_m1((-speed) as u8)
        }
    }
}