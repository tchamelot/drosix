use std::fs::File;
use std::io::Read;

use anyhow::{Context, Error, Result};

use gpio_cdev::{Chip, EventRequestFlags, LineEventHandle, LineRequestFlags};
use hal::Delay;
use hal::I2cdev;

use mpu9250::I2cDevice;
use mpu9250::{Dmp, Mpu9250};
use mpu9250::{DmpRate, MpuConfig};

pub struct Odometry {
    pub euler: [f64; 3],
    pub gyro: [f32; 3],
    pub thrust: f64,
}

pub struct Sensors {
    imu: Mpu9250<I2cDevice<I2cdev>, Dmp>,
    imu_pin: Option<LineEventHandle>,
}

impl Sensors {
    pub fn new() -> Result<Self> {
        let i2c = I2cdev::new("/dev/i2c-2").context("Opening i2c bus")?;
        let mut dmp_firmware: Vec<u8> = Vec::new();
        File::open("/lib/firmware/mpu_firmware.bin")
            .unwrap()
            .read_to_end(&mut dmp_firmware)
            .context("Reading mpu9250 firmware")?;

        let mut mpu9250 = MpuConfig::dmp().dmp_rate(DmpRate::_20Hz).build(i2c);
        mpu9250
            .init(&mut Delay, &dmp_firmware)
            .map_err(|_| Error::msg("Statring mpu9250"))?;
        mpu9250
            .reset_fifo(&mut Delay)
            .map_err(|_| Error::msg("Clearing mpu9250 data"))?;

        Ok(Self {
            imu: mpu9250,
            imu_pin: None,
        })
    }

    pub fn register_imu_event(&mut self) -> Result<&mut LineEventHandle> {
        let mut chip = Chip::new("/dev/gpiochip3").context("Opening GPIO")?;
        // 117 : gpiochip3 => 3*32 = 96. 117 - 96 = 21
        let pin = chip.get_line(21).context("Accessing IMU interrupt pin")?;
        let pin_event = pin
            .events(
                LineRequestFlags::INPUT,
                EventRequestFlags::FALLING_EDGE,
                "mpu9250",
            )
            .context("Registering IMU interrupt")?;
        self.imu_pin = Some(pin_event);
        self.imu_pin
            .as_mut()
            .ok_or_else(|| Error::msg("Registering IMU interrupt"))
    }

    pub fn handle_imu_event(&mut self) -> Result<Odometry> {
        self.imu_pin
            .as_ref()
            .and_then(|pin| pin.get_event().ok())
            .context("Accessing IMU interru pin")?;
        match self.imu.dmp_all() {
            Ok(measure) => {
                let euler = quat_to_euler(&measure.quaternion);
                let gyro = measure.gyro;
                let thrust = compute_thrust(&measure.accel, &euler);
                Ok(Odometry {
                    euler,
                    gyro,
                    thrust,
                })
            },
            Err(_) => Err(Error::msg("Reading IMU measures")),
        }
    }

    pub fn clean_imu(&mut self) -> Result<()> {
        self.imu
            .reset_fifo(&mut Delay)
            .map_err(|_| Error::msg("Reseting IMU communication"))?;
        Ok(())
    }
}

// TODO rename to taitbryan
fn quat_to_euler(q: &[f64; 4]) -> [f64; 3] {
    [
        f64::atan2(
            2.0 * (q[2] * q[3] + q[0] * q[1]),
            1.0 - 2.0 * (q[1] * q[1] + q[2] * q[2]),
        ),
        f64::asin(2.0 * (q[0] * q[2] - q[1] * q[3])),
        f64::atan2(
            2.0 * (q[1] * q[2] + q[0] * q[3]),
            1.0 - 2.0 * (q[2] * q[2] + q[3] * q[3]),
        ),
    ]
}

// Compute the thrust with the weight removed for any orientation
// angles are in radian
fn compute_thrust(accel: &[f32; 3], angles: &[f64; 3]) -> f64 {
    f64::from(accel[0]) * angles[1].sin() * -1.0
        + f64::from(accel[1]) * angles[0].sin() * angles[1].cos()
        + f64::from(accel[2]) * angles[0].cos() * angles[1].cos()
        - 9.5
}
