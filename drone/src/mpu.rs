extern crate linux_embedded_hal as hal;
extern crate mpu9250;

use std::fs::File;
use std::io::{self, Read, Write};

use hal::sysfs_gpio;
use hal::Delay;
use hal::I2cdev;

use mpu9250::Dmp;
use mpu9250::Error;
use mpu9250::Mpu9250;
use mpu9250::{DmpRate, MpuConfig};

fn normalize_quat(q: &mut [f64; 4]) {
    let sum = q.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    for x in q.iter_mut() {
        *x /= sum;
    }
}

fn quat_to_euler(q: &[f64; 4]) -> [f64; 3] {
    [
        f64::atan2(
            2.0 * (q[2] * q[3] + q[0] * q[1]),
            1.0 - 2.0 * (q[1] * q[1] + q[2] * q[2]),
        ),
        f64::asin(2.0 * (q[0] * q[2] + q[1] * q[3])),
        f64::atan2(
            2.0 * (q[1] * q[2] + q[0] * q[3]),
            1.0 - 2.0 * (q[2] * q[2] + q[3] * q[3]),
        ),
    ]
}

fn main() {
    let i2c = I2cdev::new("/dev/i2c-2").expect("unable to open /dev/i2c-2");
    let mut dmp_firmware: Vec<u8> = Vec::new();
    File::open("/lib/firmware/mpu_firmware.bin")
        .expect("unable to open MPU9250 firmware")
        .read_to_end(&mut dmp_firmware)
        .expect("unable to read MPu9250 firmware");

    let pin = sysfs_gpio::Pin::new(117);
    pin.with_exported(|| {
        pin.set_direction(sysfs_gpio::Direction::In).unwrap();
        pin.set_edge(sysfs_gpio::Edge::FallingEdge).unwrap();
        let mut event = pin.get_poller().unwrap();

        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        writeln!(&mut stdout, "  Euler angles").unwrap();

        let mut mpu_conf = *MpuConfig::<Dmp>::dmp()
            .dmp_rate(DmpRate::_50Hz)
            .dmp_features_raw_accel(false)
            .dmp_features_raw_gyro(false)
            .dmp_features_tap(true);
        let mut mpu9250 = Mpu9250::dmp(i2c, &mut Delay, &mut mpu_conf, dmp_firmware.as_slice())
            .expect("unable to load firmware");

        let mut not_ready = 0;
        loop {
            match event.poll(1000).unwrap() {
                Some(_) => match mpu9250.quaternion::<[f64; 4]>() {
                    Ok(mut measure) => {
                        normalize_quat(&mut measure);
                        let measure = quat_to_euler(&measure);
                        write!(
                            &mut stdout,
                            "\r{:>6.1} {:>6.1} {:>6.1} {}",
                            measure[0].to_degrees(),
                            measure[1].to_degrees(),
                            measure[2].to_degrees(),
                            not_ready
                        )
                        .unwrap();
                        stdout.flush().unwrap();
                        not_ready = 0;
                    }
                    Err(Error::DmpDataNotReady) => not_ready += 1,
                    Err(_) => write!(&mut stdout, "\r...").unwrap(),
                },
                None => {
                    write!(&mut stdout, "\nTimeout\n").unwrap();
                    mpu9250.reset_fifo(&mut Delay).unwrap();
                }
            }
        }
    })
    .unwrap();
}
