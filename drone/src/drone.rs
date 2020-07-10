use std::fs::File;
use std::io::{self, Read, Write};

use hal::sysfs_gpio;
use hal::Delay;
use hal::I2cdev;
use linux_embedded_hal as hal;

use mpu9250::Dmp;
use mpu9250::Error;
use mpu9250::Mpu9250;
use mpu9250::{DmpRate, MpuConfig};

use prusst::util::VolatileCell;
use prusst::{Evtout, IntcConfig, Pruss, Sysevt};

use std::sync::mpsc::Receiver;
use tokio::sync::broadcast::Sender;

use sensor::Sensors;
use controller::PruController;

#[repr(C)]
#[derive(Copy, Clone)]
struct Motors {
    pub len_ch0: VolatileCell<u32>,
    pub len_ch1: VolatileCell<u32>,
    pub len_ch2: VolatileCell<u32>,
    pub len_ch3: VolatileCell<u32>,
}

fn normalize_quat(q: &mut [f64; 4]) {
    let sum = q.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    for x in q.iter_mut() {
        *x /= sum;
    }
}

fn quat_to_euler(q: &[f64; 4]) -> [f64; 3] {
    [f64::atan2(2.0 * (q[2] * q[3] + q[0] * q[1]),
                1.0 - 2.0 * (q[1] * q[1] + q[2] * q[2])),
     f64::asin(2.0 * (q[0] * q[2] + q[1] * q[3])),
     f64::atan2(2.0 * (q[1] * q[2] + q[0] * q[3]),
                1.0 - 2.0 * (q[2] * q[2] + q[3] * q[3]))]
}

const DMP_FIRMWARE: &str = "/lib/firmware/mpu_firmware.bin";
const I2C_BUS: &str = "/dev/i2c-2";
const DMP_PIN: u32 = 117;
const PRU_FIRMWARE: &str = "/lib/firmware/servo.bin";

struct Drone<'a> {
    measures: Sender<[f64; 3]>,
    commands: Receiver<[f64; 4]>,
    imu: Mpu9250<mpu9250::I2cDevice<I2cdev>, mpu9250::Dmp>,
    pru: Pruss<'a>,
    motors: &'a Motors,
}

impl<'a> Drone<'a> {
    pub fn new(measures: Sender<[f64; 3]>,
               commands: Receiver<[f64; 4]>)
               -> Self {
        /* Init motors */
        let mut pru = Pruss::new(&IntcConfig::new_populated()).expect("Cannot start PRU subsystem");
        let motors = pru.dram1.alloc(Motors { len_ch0: VolatileCell::new(0),
                                              len_ch1: VolatileCell::new(0),
                                              len_ch2: VolatileCell::new(0),
                                              len_ch3: VolatileCell::new(0) });
        let irq = pru.intc.register_irq(Evtout::E0);
        let mut pru_firmware =
            File::open(PRU_FIRMWARE).expect("Cannot open PRU firmware");
        unsafe {
            pru.pru1
               .load_code(&mut pru_firmware)
               .expect("Cannot load PRU firmware")
               .run();
        }
        irq.wait();
        pru.intc.clear_sysevt(Sysevt::S19);
        pru.intc.enable_host(Evtout::E0);

        /* init IMU */
        let i2c = I2cdev::new(I2C_BUS).expect("Cannot open I2C bus");
        let mut dmp_firmware: Vec<u8> = Vec::new();
        File::open(DMP_FIRMWARE).expect("Cannot open MPU9250 firmware")
                                .read_to_end(&mut dmp_firmware)
                                .expect("Cannot read MPu9250 firmware");

        let mut mpu_conf =
            *MpuConfig::<Dmp>::dmp().dmp_rate(DmpRate::_50Hz)
                                    .dmp_features_raw_accel(false)
                                    .dmp_features_raw_gyro(false)
                                    .dmp_features_tap(true);
        let imu = Mpu9250::dmp(i2c, &mut Delay, &mut mpu_conf, dmp_firmware.as_slice()).expect("Cannot load DMP firmware");

        Self { measures,
               commands,
               imu,
               pru,
               motors }
    }

    pub fn run(&mut self) {
        let pin = sysfs_gpio::Pin::new(DMP_PIN);
        pin.with_exported(|| {
            pin.set_direction(sysfs_gpio::Direction::In).unwrap();
            pin.set_edge(sysfs_gpio::Edge::FallingEdge).unwrap();
            let mut event = pin.get_poller().unwrap();
            loop {
                match event.poll(1000).unwrap() {
                Some(_) => match self.imu.all().unwrap() {
                    _ => (),
                },
                    None => (),
            }
           });
    }
}

pub fn drone(sender: Sender<[f64; 3]>, receiver: Receiver<[f64; 4]>) {
    let mut pruss = match Pruss::new(&IntcConfig::new_populated()) {
        Ok(p) => p,
        Err(e) => match e {
            prusst::Error::AlreadyInstantiated => {
                panic!("You can't instantiate more than one `Pruss` object at a time.")
            }
            prusst::Error::PermissionDenied => panic!(
                "You do not have permission to access the PRU subsystem: \
                           maybe you should run this program as root?"
            ),
            prusst::Error::DeviceNotFound => panic!(
                "The PRU subsystem could not be found: are you sure the `uio_pruss` \
                           module is loaded and supported by your kernel?"
            ),
            prusst::Error::OtherDeviceError => panic!(
                "An unidentified problem occured with the PRU subsystem: \
                           do you have a valid overlay loaded?"
            ),
        },
    };

    let servo = pruss.dram1.alloc(Motors { len_ch0: VolatileCell::new(0),
                                           len_ch1: VolatileCell::new(0),
                                           len_ch2: VolatileCell::new(0),
                                           len_ch3: VolatileCell::new(0) });
    let irq = pruss.intc.register_irq(Evtout::E0);

    let mut pru_binary = File::open("/lib/firmware/servo.bin").unwrap();
    unsafe {
        pruss.pru1.load_code(&mut pru_binary).unwrap().run();
    }

    irq.wait();
    pruss.intc.clear_sysevt(Sysevt::S19);
    pruss.intc.enable_host(Evtout::E0);

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

           let mut mpu_conf =
               MpuConfig::<Dmp>::dmp().dmp_rate(DmpRate::_50Hz).build(i2c);
           mpu9250.init(&mut Delay, &dmp_firmware)
                  .expect("Unable to load firmware");
           loop {
               match event.poll(1000).unwrap() {
                   Some(_) => match mpu9250.dmp_all() {
                       Ok(mut measure) => {
                           let euler = quat_to_euler(&measure.quaternion);
                           sender.send(euler);
                           if let Some(cmd) = receiver.try_iter().last() {
                               println!("received {:?}", cmd);
                               servo.len_ch0
                                    .set(((cmd[0] + 50.0) / 100.0 * 14_285.0
                                          + 14_285.0)
                                         as u32);
                               servo.len_ch1
                                    .set(((cmd[1] + 50.0) / 100.0 * 14_285.0
                                          + 14_285.0)
                                         as u32);
                               servo.len_ch2
                                    .set(((cmd[2] + 50.0) / 100.0 * 14_285.0
                                          + 14_285.0)
                                         as u32);
                               servo.len_ch3
                                    .set(((cmd[3] + 50.0) / 100.0 * 14_285.0
                                          + 14_285.0)
                                         as u32);
                           }
                       },
                       Err(_) => (),
                   },
                   None => {
                       mpu9250.reset_fifo(&mut Delay).unwrap();
                   },
               }
           }
       })
       .unwrap();
}
