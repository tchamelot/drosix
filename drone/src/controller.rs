use prusst::{Evtout, IntcConfig, Pruss, Sysevt, Channel, Host};
use prusst::util::VolatileCell;

use std::fs::File;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::thread;

#[repr(C)]
#[derive(Copy, Clone)]
struct Pid {
    pub kp: i32,
    pub ki: i32,
    pub kd: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Controller {
    pub input: VolatileCell<i32>,
    pub output: VolatileCell<u32>,
    pub pid: VolatileCell<Pid>,
    pub cycle: VolatileCell<u32>,
    pub stall: VolatileCell<u32>,
}

fn main() {
    let mut int_conf = IntcConfig::new_empty();
    int_conf.map_sysevts_to_channels(
            &[  (Sysevt::S17, Channel::C0),
                (Sysevt::S18, Channel::C0),
                (Sysevt::S20, Channel::C0),
                (Sysevt::S16, Channel::C1),
                (Sysevt::S21, Channel::C1),
                (Sysevt::S19, Channel::C2),
                (Sysevt::S31, Channel::C3)]
        );
    int_conf.map_channels_to_hosts(
            &[  (Channel::C0, Host::Pru0),
                (Channel::C1, Host::Pru1),
                (Channel::C2, Host::Evtout0),
                (Channel::C3, Host::Evtout1)]
        );
    int_conf.auto_enable_sysevts();
    int_conf.auto_enable_hosts();

    let mut pruss = match Pruss::new(&int_conf) {
        Ok(p) => p,
        Err(e) => match e {
            prusst::Error::AlreadyInstantiated => panic!("Pruss: already in use"), 
            prusst::Error::PermissionDenied => panic!( "Pruss: permission denied "),
            prusst::Error::DeviceNotFound => panic!( "Pruss: char device not found"),
            prusst::Error::OtherDeviceError => panic!( "Pruss: unidentified problem occured"),
        },
    };

    let irq0 = pruss.intc.register_irq(Evtout::E0);
    let irq1 = pruss.intc.register_irq(Evtout::E1);

    let controller = pruss.dram2.alloc(Controller {
        input: VolatileCell::new(1),
        output: VolatileCell::new(179999),
        pid: VolatileCell::new(Pid {kp: 1, ki: 0, kd: 0}),
        cycle: VolatileCell::new(0),
        stall: VolatileCell::new(0),
    });

    // Open and load a PRU binary.
    let mut controller_bin = File::open("/lib/firmware/controller.bin").unwrap();
    let mut motor_bin = File::open("/lib/firmware/motor.bin").unwrap();
    unsafe {
        pruss.pru0.load_code(&mut controller_bin).unwrap().run();
    }
    unsafe {
        pruss.pru1.load_code(&mut motor_bin).unwrap().run();
    }

    irq0.wait();
    pruss.intc.clear_sysevt(Sysevt::S19);
    pruss.intc.enable_host(Evtout::E0);
    println!("Controller started");

    let mut now = Instant::now();
    
    let mut input = 215999;
    for _ in 0..10 {
        controller.input.set(input);
        pruss.intc.send_sysevt(Sysevt::S18);

        irq1.wait();
        pruss.intc.clear_sysevt(Sysevt::S31);
        pruss.intc.enable_host(Evtout::E1);
        println!("running {:<+3} -> {:<+3}: {:<2} - {:<2} - {:<2}",
            input,
            controller.output.get(),
            now.elapsed().as_millis(),
            controller.cycle.get(),
            controller.stall.get());
        /* input += 1; */
        now = Instant::now()
    }

    pruss.intc.send_sysevt(Sysevt::S17);
    pruss.intc.send_sysevt(Sysevt::S16);

    irq0.wait();
    pruss.intc.clear_sysevt(Sysevt::S19);
    println!("Controller stoped");
}
