use prusst::util::VolatileCell;
use prusst::{Channel, Evtout, EvtoutIrq, Host, IntcConfig, Pruss, Sysevt};

use anyhow::{Context, Result};

use std::fs::File;
use std::thread;
use std::time;

/// PID controller parameters
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Pid {
    /// PID proportional gain
    pub kp: f32,
    /// PID integral gain
    pub ki: f32,
    /// PID derivative gain 1
    pub kd1: f32,
    /// PID derivative gain 2
    pub kd2: f32,
    /// PID minimum value
    pub min: f32,
    /// PID maximum value
    pub max: f32,
}

impl Default for Pid {
    fn default() -> Self {
        Pid {
            kp: 0.0,
            ki: 0.0,
            kd1: 0.0,
            kd2: 0.0,
            min: 0.0,
            max: 0.0,
        }
    }
}

/// Shared memory between the Cortex-A8 and the two PRUs.
/// This structure should only be allocated once by the PRU controller.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Controller {
    /// PID controller inputs: Roll, Pitch, Yaw, Thrust, angular velocities
    pub input: [VolatileCell<f32>; 7],
    /// PID controller outputs: Motor[1-4] duty cycles
    pub output: [VolatileCell<u32>; 4],
    /// PID controller parameters
    pub pid: [VolatileCell<Pid>; 7],
    /// For debug purpose: number of cycles recorded by a PRU
    pub cycle: VolatileCell<u32>,
    /// For debug purpose: number of stall cycles recorded by a PRU
    pub stall: VolatileCell<u32>,
}

impl Default for Controller {
    fn default() -> Self {
        Controller {
            input: [VolatileCell::new(0.0); 7],
            output: [VolatileCell::new(199_999); 4],
            pid: [VolatileCell::new(Default::default()); 7],
            cycle: VolatileCell::new(0),
            stall: VolatileCell::new(0),
        }
    }
}

const EVENT_MAP: [(Sysevt, Channel); 4] = [
    (Sysevt::S20, Channel::C2), /* MOTOR_STATUS */
    (Sysevt::S16, Channel::C1), /* MOTOR_STOP */
    (Sysevt::S21, Channel::C1), /* PID_OUTPUT */
    (Sysevt::S31, Channel::C3), /* DEBUG */
];
const CHANNEL_MAP: [(Channel, Host); 3] = [
    (Channel::C1, Host::Pru1),    /* PRU1 */
    (Channel::C2, Host::Evtout0), /* HOST */
    (Channel::C3, Host::Evtout1), /* HOST_DEBUG */
];

fn main() -> Result<()> {
    let mut int_conf = IntcConfig::new_empty();
    int_conf.map_sysevts_to_channels(&EVENT_MAP);
    int_conf.map_channels_to_hosts(&CHANNEL_MAP);
    int_conf.auto_enable_sysevts();
    int_conf.auto_enable_hosts();
    let mut pru = Pruss::new(&int_conf).context("Intanciating PRUSS")?;

    // Init PRU shared mem
    let controller = pru.dram2.alloc(Controller::default());

    let status_evt = pru.intc.register_irq(Evtout::E0);
    let debug_evt = pru.intc.register_irq(Evtout::E1);

    let mut motor_bin = File::open("/lib/firmware/motor.bin")
        .context("Opening ESC controller firmware")?;
    let mut motor_code = pru
        .pru1
        .load_code(&mut motor_bin)
        .context("Loading ESC controller firmware")?;
    unsafe {
        motor_code.run();
    }

    status_evt.wait();
    pru.intc.clear_sysevt(Sysevt::S20);
    pru.intc.enable_host(Evtout::E0);
    println!("Motor started");
    thread::sleep(time::Duration::from_secs(1));

    for i in 0..4 {
        thread::sleep(time::Duration::from_secs(1));
        println!("Starting motor {}", i);
        controller.output[i].set(240_000);
        pru.intc.send_sysevt(Sysevt::S21);
        thread::sleep(time::Duration::from_secs(3));
        controller.output[i].set(199_999);
        pru.intc.send_sysevt(Sysevt::S21);
    }

    println!("Stopping motor...");
    pru.intc.send_sysevt(Sysevt::S16);
    status_evt.wait();
    println!("Motor stopped");
    return Ok(());
}
