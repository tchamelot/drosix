use std::fs;
use std::os::unix::io::{AsRawFd, RawFd};

use anyhow::{Context, Result};

use hal::gpio_cdev::{Chip, EventRequestFlags, LineEventHandle, LineRequestFlags};
use hal::i2cdev::linux::LinuxI2CError;
use hal::Delay;
use hal::I2cdev;

use mpu9250::I2cDevice;
use mpu9250::{Dmp, Mpu9250};
use mpu9250::{DmpRate, MpuConfig};

use crate::types::{Angles, Odometry};

#[derive(Debug)]
pub enum Error {
    NotCalibarated,
    NotAvailable,
    Mpu9250(mpu9250::Error<mpu9250::I2CError<LinuxI2CError>>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotCalibarated => write!(f, "Sensors not calibrated"),
            Self::NotAvailable => write!(f, "Sensors data not available"),
            Self::Mpu9250(x) => write!(f, "Internal IMU error: {:?}", x),
        }
    }
}

impl std::error::Error for Error {}

pub struct Sensors {
    imu: Mpu9250<I2cDevice<I2cdev>, Dmp>,
    imu_pin: LineEventHandle,
    imu_calibrated: bool,
}

impl Sensors {
    /// Initiates all the sensors:
    /// - IMU: MPU9250
    pub fn new() -> Result<Self> {
        let i2c = I2cdev::new("/dev/i2c-2").context("Opening i2c bus")?;

        let mut mpu_config = MpuConfig::dmp();
        mpu_config
            .dmp_rate(DmpRate::_100Hz)
            .dmp_features_raw_gyro(true)
            .dmp_features_raw_accel(true)
            .dmp_features_quat6(true)
            .dmp_features_gyro_auto_calibrate(true);

        let dmp_firmware = fs::read("/lib/firmware/mpu_firmware.bin")?;

        // 117 : gpiochip3 => 3*32 = 96. 117 - 96 = 21
        let pin_event = Chip::new("/dev/gpiochip3")
            .and_then(|mut chip| chip.get_line(21))
            .and_then(|line| line.events(LineRequestFlags::INPUT, EventRequestFlags::FALLING_EDGE, "mpu9250"))
            .context("Registering IMU interrupt")?;

        let mpu9250 = Mpu9250::dmp(i2c, &mut Delay, &mut mpu_config, &dmp_firmware).map_err(|e| Error::Mpu9250(e))?;

        Ok(Self {
            imu: mpu9250,
            imu_pin: pin_event,
            imu_calibrated: false,
        })
    }

    pub fn imu_event(&mut self) -> RawFd {
        self.imu_pin.as_raw_fd()
    }

    pub fn handle_imu_event(&mut self) -> Result<Odometry> {
        self.imu_pin.get_event().context("Accessing IMU interrupt pin")?;
        match self.imu.dmp_all::<[f32; 3], [f64; 4]>() {
            Ok(measure) => {
                let attitude = quat_to_angles(&measure.quaternion.unwrap());
                let gyro = measure.gyro.unwrap();
                let thrust = compute_thrust(&measure.accel.unwrap(), &attitude);
                Ok(Odometry {
                    attitude,
                    rate: Angles {
                        roll: gyro[2],
                        pitch: gyro[1],
                        yaw: gyro[0],
                    },
                    thrust,
                })
            },
            Err(mpu9250::Error::DmpDataNotReady) => Err(Error::NotAvailable.into()),
            Err(x) => Err(Error::Mpu9250(x).into()),
        }
    }

    /// Reset the IMU internal state keeping the same config
    pub fn clean_imu(&mut self) -> Result<()> {
        self.imu.reset_fifo(&mut Delay).map_err(|e| Error::Mpu9250(e).into())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polling::Poller;
    use mio::{Interest, Token};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn test_sensors_config() {
        // Setup interrupt infrastructure
        let mut poller = Poller::new(8).unwrap();
        const SENSORS: Token = Token(0);

        let mut sensors = Sensors::new().context("Cannot start sensors").unwrap();
        poller.register(&sensors.imu_event(), SENSORS, Interest::READABLE).unwrap();

        let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
        if events.is_empty() {
            panic!("Sensors did not start correctly");
        }

        sensors.handle_imu_event().unwrap();

        // Check interrupt frequency is 100hZ
        let start = Instant::now();
        for _ in 0..100 {
            let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
            if events.is_empty() {
                panic!("Sensors time-out reached");
            }
            sensors.handle_imu_event().unwrap();
        }
        let period = start.elapsed().as_millis() / 100;
        if period > 11 {
            panic!("Sensors interrupt to slow: period is {}", period / 100);
        }
        if period < 9 {
            panic!("Sensors interrupt to fast: period is {}", period / 100);
        }
    }

    // Cannot reproduce case where the flight controller loop does not get any interrupt from
    // sensors
    #[test]
    #[ignore]
    fn test_sensors_reset() {
        // Setup interrupt infrastructure
        let mut poller = Poller::new(8).unwrap();
        const SENSORS: Token = Token(0);

        let mut sensors = Sensors::new().context("Cannot start sensors").unwrap();
        poller.register(&sensors.imu_event(), SENSORS, Interest::READABLE).unwrap();

        // Let several interrupts pass to see if we get the next one
        thread::sleep(Duration::from_secs(5));
        let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
        if events.is_empty() {
            panic!("Sensors stoped sending interrupt");
        }
    }
}
