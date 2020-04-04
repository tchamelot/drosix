extern crate prusst;

use prusst::util::VolatileCell;
use prusst::{Evtout, IntcConfig, Pruss, Sysevt};

use std::fs::File;
use std::io::{self, Write};

#[repr(C)]
#[derive(Copy, Clone)]
struct Servo {
    pub len_ch1: VolatileCell<u32>,
    pub len_ch2: VolatileCell<u32>,
    pub len_ch3: VolatileCell<u32>,
    pub len_ch4: VolatileCell<u32>,
}

fn main() {
    // Get a view of the PRU subsystem.
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

    println!("Allocate ctrl {:?}", pruss.dram1.begin());
    let servo = pruss.dram1.alloc(Servo {
        len_ch1: VolatileCell::new(0),
        len_ch2: VolatileCell::new(0),
        len_ch3: VolatileCell::new(0),
        len_ch4: VolatileCell::new(0),
    });

    let irq = pruss.intc.register_irq(Evtout::E0);

    // Open and load a PRU binary.
    let mut pru_binary = File::open("/lib/firmware/servo.bin").unwrap();
    unsafe {
        pruss.pru1.load_code(&mut pru_binary).unwrap().run();
    }

    irq.wait();
    pruss.intc.clear_sysevt(Sysevt::S19);
    pruss.intc.enable_host(Evtout::E0);
    println!("starting");

    loop {
        let len: u32 = get_input("Power [%]", 0.0, 1.0, -0.1);
        servo.len_ch1.set(len);
        servo.len_ch2.set(len);
        servo.len_ch3.set(len);
        servo.len_ch4.set(len);

        if len == 0 {
            pruss.intc.send_sysevt(Sysevt::S21);
            irq.wait();
            pruss.intc.clear_sysevt(Sysevt::S19);
            break;
        }
    }

    println!("Goodbye");
}

fn get_input(prompt: &str, min: f32, max: f32, stop: f32) -> u32 {
    loop {
        print!("{} ({}-{}) or  {} to stop: ", prompt, min, max, stop);
        io::stdout().flush().unwrap();

        let mut val = String::new();
        io::stdin()
            .read_line(&mut val)
            .expect("failed to read input");

        if let Ok(val) = val.trim().parse::<f32>() {
            if val >= min && val <= max {
                return (val * 14_285.0 + 14_285.0) as u32;
            } else if val == stop {
                return 0;
            } else {
                println!("the input should be between {} and {}", min, max);
            }
        } else {
            println!("not a number");
        }
    }
}
