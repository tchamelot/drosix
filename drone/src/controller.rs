use prusst::util::VolatileCell;
use prusst::{Channel, Evtout, EvtoutIrq, Host, IntcConfig, Pruss, Sysevt};

use anyhow::{Context, Result};

use std::fs::File;

/// PID controller parameters
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Pid {
    /// PID proportional gain
    pub kp: i32,
    /// PID integral gain
    pub ki: i32,
    /// PID derivative gain
    pub kd: i32,
}

impl Default for Pid {
    fn default() -> Self {
        Pid {
            kp: 0,
            ki: 0,
            kd: 0,
        }
    }
}

/// Shared memory between the Cortex-A8 and the two PRUs.
/// This structure should only be allocated once by the PRU controller.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Controller {
    /// PID controller inputs: Roll, Pitch, Yaw, Thrust, angular velocities
    pub input: [VolatileCell<i32>; 7],
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
            input: [VolatileCell::new(-20_000); 7],
            output: [VolatileCell::new(179_999); 4],
            pid: [VolatileCell::new(Default::default()); 7],
            cycle: VolatileCell::new(0),
            stall: VolatileCell::new(0),
        }
    }
}

const EVENT_MAP: [(Sysevt, Channel); 9] = [
    (Sysevt::S17, Channel::C0), /* CONTROLLER_STOP */
    (Sysevt::S18, Channel::C0), /* PID_NEW_DATA */
    (Sysevt::S20, Channel::C0), /* MOTOR_STATUS */
    (Sysevt::S16, Channel::C1), /* MOTOR_STOP */
    (Sysevt::S21, Channel::C1), /* PID_OUTPUT */
    (Sysevt::S22, Channel::C0), /* SET_ARMED */
    (Sysevt::S23, Channel::C0), /* CLEAR_ARMED */
    (Sysevt::S19, Channel::C2), /* CONTROLLER_STATUS */
    (Sysevt::S31, Channel::C3), /* DEBUG */
];
const CHANNEL_MAP: [(Channel, Host); 4] = [
    (Channel::C0, Host::Pru0),    /* PRU0 */
    (Channel::C1, Host::Pru1),    /* PRU1 */
    (Channel::C2, Host::Evtout0), /* HOST */
    (Channel::C3, Host::Evtout1), /* HOST_DEBUG */
];

/// Interface between the Linux part and the PRUs subsystems.
pub struct PruController<'a> {
    pru: Pruss<'a>,
    shared_mem: &'a mut Controller,
    status_evt: EvtoutIrq,
    debug_evt: EvtoutIrq,
    running: bool,
}

impl<'a> PruController<'a> {
    /// Create a new instance of the PRU controller with default PID parameters
    pub fn new() -> Result<Self> {
        // Init PRU events
        let mut int_conf = IntcConfig::new_empty();
        int_conf.map_sysevts_to_channels(&EVENT_MAP);
        int_conf.map_channels_to_hosts(&CHANNEL_MAP);
        int_conf.auto_enable_sysevts();
        int_conf.auto_enable_hosts();
        let mut pru = Pruss::new(&int_conf).context("Intanciating PRUSS")?;

        // Init PRU shared mem
        let controller =
            pru.dram2.alloc(Controller::default()) as *mut Controller;

        let status_evt = pru.intc.register_irq(Evtout::E0);
        let debug_evt = pru.intc.register_irq(Evtout::E1);

        Ok(PruController {
            pru,
            shared_mem: unsafe { &mut *(controller.clone()) },
            status_evt,
            debug_evt,
            running: false,
        })
    }

    /// Start the PRU (load and launch firmwares)
    pub fn start(&mut self) -> Result<()> {
        // Load PRU code
        let mut controller_bin = File::open("/lib/firmware/controller.bin")
            .context("Opening PID controller firmware")?;
        let mut motor_bin = File::open("/lib/firmware/motor.bin")
            .context("Opening ESC controller firmware")?;
        let mut contoller_code = self
            .pru
            .pru0
            .load_code(&mut controller_bin)
            .context("Loading PID controller firmware")?;
        let mut motor_code = self
            .pru
            .pru1
            .load_code(&mut motor_bin)
            .context("Loading ESC controller firmware")?;
        unsafe {
            contoller_code.run();
            motor_code.run();
        }
        Ok(())
    }

    /// Return a polling event linked to PRU status change interrupt
    pub fn register_pru_evt(&mut self) -> &mut EvtoutIrq {
        &mut self.status_evt
    }

    /// Return a polling event linked to PRU debug interrupt
    pub fn register_pru_debug(&mut self) -> &mut EvtoutIrq {
        &mut self.debug_evt
    }

    /// Handle a status change event
    /// Return true if the flight controller is running
    /// Return false if the flight controller has stopped whether because of
    /// an error or because of the natural ending of the firmware
    pub fn handle_event(&mut self) -> bool {
        self.pru.intc.clear_sysevt(Sysevt::S19);
        self.pru.intc.enable_host(Evtout::E0);
        if !self.running {
            self.running = true;
        } else {
            self.running = false;
        }

        self.running
    }

    /// Handle a debug event by printing the shared memory state
    pub fn handle_debug(&mut self) {
        self.pru.intc.clear_sysevt(Sysevt::S31);
        self.pru.intc.enable_host(Evtout::E1);
        //dbg!(self.shared_mem.output);
        dbg!(self.shared_mem.output[0].get());
    }

    /// Send new values to the PID controller
    /// New values will be processed only if the motor are armed
    pub fn set_pid_inputs(&mut self, inputs: [i32; 4]) {
        self.shared_mem.input[0].set(inputs[0]);
        self.shared_mem.input[1].set(inputs[1]);
        self.shared_mem.input[2].set(inputs[2]);
        self.shared_mem.input[3].set(inputs[3]);
        self.pru.intc.send_sysevt(Sysevt::S18);
    }

    /// Arm the motor making the PID controller start
    pub fn set_armed(&mut self) {
        self.pru.intc.send_sysevt(Sysevt::S22);
    }

    /// Disarm the motor making the PID controller stop
    pub fn clear_armed(&mut self) {
        self.pru.intc.send_sysevt(Sysevt::S23);
    }

    /// Stop the PRU subsystems.
    /// The stop is effective after receiving a new status change event.
    /// The `handle_event` function should return false after this event.
    pub fn stop(&mut self) {
        if self.running {
            self.pru.intc.send_sysevt(Sysevt::S17);
        }
    }
}

impl<'a> Drop for PruController<'a> {
    fn drop(&mut self) {
        self.stop()
    }
}
