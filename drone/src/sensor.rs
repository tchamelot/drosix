use std::fs::File;
use std::io::Read;
use std::os::unix::io::{AsRawFd, RawFd};

use anyhow::{Context, Error, Result};

use gpio_cdev::{Chip, EventRequestFlags, LineEventHandle, LineRequestFlags};
use hal::Delay;
use hal::I2cdev;

use mpu9250::I2cDevice;
use mpu9250::{Dmp, Mpu9250};
use mpu9250::{DmpRate, MpuConfig};

use crate::types::{Angles, Odometry};

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

        let mut mpu9250 = MpuConfig::dmp().dmp_rate(DmpRate::_100Hz).build(i2c);
        mpu9250.init(&mut Delay, &dmp_firmware).map_err(|_| Error::msg("Statring mpu9250"))?;
        mpu9250.reset_fifo(&mut Delay).map_err(|_| Error::msg("Clearing mpu9250 data"))?;

        Ok(Self {
            imu: mpu9250,
            imu_pin: None,
        })
    }

    pub fn register_imu_event(&mut self) -> Result<RawFd> {
        let mut chip = Chip::new("/dev/gpiochip3").context("Opening GPIO")?;
        // 117 : gpiochip3 => 3*32 = 96. 117 - 96 = 21
        let pin = chip.get_line(21).context("Accessing IMU interrupt pin")?;
        let pin_event = pin
            .events(LineRequestFlags::INPUT, EventRequestFlags::FALLING_EDGE, "mpu9250")
            .context("Registering IMU interrupt")?;
        self.imu_pin = Some(pin_event);
        self.imu_pin.as_ref().context("Unreachable").map(|x| x.as_raw_fd())
    }

    pub fn handle_imu_event(&mut self) -> Result<Odometry> {
        self.imu_pin.as_mut().and_then(|pin| pin.get_event().ok()).context("Accessing IMU interru pin")?;
        match self.imu.dmp_all() {
            Ok(measure) => {
                let attitude = quat_to_angles(&measure.quaternion);
                let gyro = measure.gyro;
                let thrust = compute_thrust(&measure.accel, &attitude);
                Ok(Odometry {
                    attitude,
                    rate: Angles {
                        roll: gyro[1],
                        pitch: gyro[0],
                        yaw: gyro[2],
                    },
                    thrust,
                })
            },
            Err(_) => Err(Error::msg("Reading IMU measures")),
        }
    }

    pub fn clean_imu(&mut self) -> Result<()> {
        self.imu.reset_fifo(&mut Delay).map_err(|_| Error::msg("Reseting IMU communication"))?;
        Ok(())
    }
}

fn quat_to_angles(q: &[f64; 4]) -> Angles {
    // roll and pitch are swapped due to hardware positioning of the IMU
    Angles {
        roll: f64::asin(2.0 * (q[0] * q[2] - q[1] * q[3])) as f32,
        pitch: f64::atan2(2.0 * (q[2] * q[3] + q[0] * q[1]), 1.0 - 2.0 * (q[1] * q[1] + q[2] * q[2])) as f32,
        yaw: f64::atan2(2.0 * (q[1] * q[2] + q[0] * q[3]), 1.0 - 2.0 * (q[2] * q[2] + q[3] * q[3])) as f32,
    }
}

// Compute the thrust with the weight removed for any orientation angles are in radian
// TODO not used yet. Needs verification and testing
fn compute_thrust(accel: &[f32; 3], angles: &Angles) -> f32 {
    f32::from(accel[0]) * angles.roll.sin() * -1.0
        + f32::from(accel[1]) * angles.pitch.sin() * angles.roll.cos()
        + f32::from(accel[2]) * angles.pitch.cos() * angles.roll.cos()
        - 9.5
}
