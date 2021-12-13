use anyhow::{Context, Result};
use prusst::util::VolatileCell;
use prusst::{Channel, Evtout, EvtoutIrq, Host, IntcConfig, Pruss, Sysevt};
use std::os::unix::io::{AsRawFd, RawFd};

use rkyv::{Archive, Deserialize, Serialize};

use std::fs::File;

const MOTORS_FW: &str = "/lib/firmware/motor.bin";
const PID_FW: &str = "/lib/firmware/controller.bin";

/// PID controller parameters
#[repr(C)]
#[derive(Archive, Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Pid {
    /// PID input gains
    pub a: [f32; 3],
    /// PID output gains
    pub b: [f32; 2],
}

impl Default for Pid {
    fn default() -> Self {
        Pid {
            a: [0.; 3],
            b: [0.; 2],
        }
    }
}

/// Shared memory between the Cortex-A8 and the two PRUs.
/// This structure should only be allocated once by the PRU controller.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PruSharedMem {
    /// PID controller inputs: Roll, Pitch, Yaw, Thrust, angular velocities
    pub pid_input: [VolatileCell<f32>; 7],
    /// PID controller outputs: Motor[1-4] duty cycles
    pub pid_output: [VolatileCell<u32>; 4],
    /// PID controller parameters
    pub pid_config: [VolatileCell<Pid>; 7],
    /// For debug purpose: indicates which event should trigger a debug event
    pub debug_location: VolatileCell<u32>,
    /// For debug purpose: position PID outputs
    pub p_pid: [VolatileCell<f32>; 3],
    /// For debug purpose: speed PID outputs
    pub v_pid: [VolatileCell<f32>; 3],
    /// For debug purpose: number of cycles recorded by a PRU
    pub cycle: VolatileCell<u32>,
    /// For debug purpose: number of stall cycles recorded by a PRU
    pub stall: VolatileCell<u32>,
}

impl Default for PruSharedMem {
    fn default() -> Self {
        PruSharedMem {
            pid_input: [VolatileCell::new(0.0); 7],
            pid_output: [VolatileCell::new(179_999); 4],
            pid_config: [VolatileCell::new(Default::default()); 7],
            debug_location: VolatileCell::new(0),
            p_pid: [VolatileCell::new(0.0); 3],
            v_pid: [VolatileCell::new(0.0); 3],
            cycle: VolatileCell::new(0),
            stall: VolatileCell::new(0),
        }
    }
}

pub const DEBUG_PID_LOOP: u32 = 1 << 0;
pub const DEBUG_PID_NEW_DATA: u32 = 1 << 1;
pub const DEBUG_PWM_STEP: u32 = 1 << 2;

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
pub struct Controller<'a> {
    pru: Pruss<'a>,
    shared_mem: &'a mut PruSharedMem,
    status_evt: EvtoutIrq,
    debug_evt: EvtoutIrq,
    running: bool,
}

impl<'a> Controller<'a> {
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
        let shared_mem = pru.dram2.alloc(PruSharedMem::default());
        // Use transmute to extend the lifetime. It is ok because pru has the
        // lifetime 'a and the controller ref has the same lifetime.
        // Moreover, the ref is not visible outside of the Controller
        let shared_mem = unsafe { std::mem::transmute(shared_mem) };

        let status_evt = pru.intc.register_irq(Evtout::E0);
        let debug_evt = pru.intc.register_irq(Evtout::E1);

        Ok(Controller {
            pru,
            shared_mem,
            status_evt,
            debug_evt,
            running: false,
        })
    }

    pub fn set_pid_configs(&mut self, pid_configs: [Pid; 7]) {
        // equivalent to memcpy
        unsafe {
            self.shared_mem.pid_config = std::mem::transmute(pid_configs);
        }
    }

    /// Start the PRU (load and launch firmwares)
    pub fn start(&mut self) -> Result<()> {
        // Load PRU code
        let mut pid_fw =
            File::open(PID_FW).context("Opening PID controller firmware")?;
        let mut motor_fw =
            File::open(MOTORS_FW).context("Opening ESC controller firmware")?;
        let mut contoller_code = self
            .pru
            .pru0
            .load_code(&mut pid_fw)
            .context("Loading PID controller firmware")?;
        let mut motor_code = self
            .pru
            .pru1
            .load_code(&mut motor_fw)
            .context("Loading ESC controller firmware")?;
        unsafe {
            contoller_code.run();
            motor_code.run();
        }
        Ok(())
    }

    /// Return a polling event linked to PRU status change interrupt
    pub fn register_pru_evt(&self) -> RawFd {
        self.status_evt.as_raw_fd()
    }

    /// Return a polling event linked to PRU debug interrupt
    pub fn register_pru_debug(&self) -> RawFd {
        self.debug_evt.as_raw_fd()
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

    /// Handle a debug event and return the current shared mem state
    pub fn handle_debug(&mut self) -> PruSharedMem {
        self.pru.intc.clear_sysevt(Sysevt::S31);
        self.pru.intc.enable_host(Evtout::E1);
        *self.shared_mem
    }

    /// Set speed for the given motor
    pub fn set_motor_speed(&mut self, motor: usize, speed: u32) -> Result<()> {
        if motor > 3 {
            return Err(())
                .ok()
                .context(format!("Cannot set speed for motor {}", motor));
        }
        if speed < 199_999 || speed > 299_999 {
            return Err(()).ok().context(format!(
                "Cannot set motor {} speed to {} range is [199999;299999]",
                motor, speed
            ));
        }
        self.shared_mem.pid_output[motor].set(speed);
        self.pru.intc.send_sysevt(Sysevt::S21);
        Ok(())
    }

    /// Send new values to the PID controller
    /// New values will be processed only if the motor are armed
    pub fn set_pid_inputs(&mut self, inputs: [f32; 7]) {
        self.shared_mem.pid_input[0].set(inputs[0]);
        self.shared_mem.pid_input[1].set(inputs[1]);
        self.shared_mem.pid_input[2].set(inputs[2]);
        self.shared_mem.pid_input[3].set(inputs[3]);
        self.shared_mem.pid_input[4].set(inputs[4]);
        self.shared_mem.pid_input[5].set(inputs[5]);
        self.shared_mem.pid_input[6].set(inputs[6]);
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

    pub fn set_debug(&mut self, dbg: u32) {
        let dbg = self.shared_mem.debug_location.get() | dbg;
        self.shared_mem.debug_location.set(dbg);
    }

    pub fn reset_debug(&mut self, dbg: u32) {
        let dbg = self.shared_mem.debug_location.get() & !dbg;
        self.shared_mem.debug_location.set(dbg);
    }

    /// Stop the PRU subsystems.
    /// The stop is effective after receiving a new status change event.
    /// The `handle_event` function should return false after this event.
    pub fn stop(&mut self) {
        if self.running {
            self.pru.intc.send_sysevt(Sysevt::S16);
        }
    }
}

impl<'a> Drop for Controller<'a> {
    fn drop(&mut self) {
        self.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mio::{Events, Interest, Poll, Token};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_motors() {
        const MOTOR_EVENT_MAP: [(Sysevt, Channel); 4] = [
            (Sysevt::S20, Channel::C2), /* MOTOR_STATUS */
            (Sysevt::S16, Channel::C1), /* MOTOR_STOP */
            (Sysevt::S21, Channel::C1), /* PID_OUTPUT */
            (Sysevt::S31, Channel::C3), /* DEBUG */
        ];
        const MOTOR_CHANNEL_MAP: [(Channel, Host); 3] = [
            (Channel::C1, Host::Pru1),    /* PRU1 */
            (Channel::C2, Host::Evtout0), /* HOST */
            (Channel::C3, Host::Evtout1), /* HOST_DEBUG */
        ];

        let mut controller = Controller::new().unwrap();

        // Configuring PRU subsystem
        let mut conf = IntcConfig::new_empty();
        conf.map_sysevts_to_channels(&MOTOR_EVENT_MAP);
        conf.map_channels_to_hosts(&MOTOR_CHANNEL_MAP);
        conf.auto_enable_sysevts();
        conf.auto_enable_hosts();

        controller.pru.intc.map_interrupts(&conf);

        // Interruption event catching
        let mut poll = Poll::new().context("Creating event poller").unwrap();
        let mut events = Events::with_capacity(8);
        const PRU_STATUS: Token = Token(0);
        poll.registry()
            .register(
                controller.register_pru_evt(),
                PRU_STATUS,
                Interest::READABLE,
            )
            .context("Regitering pru status event")
            .unwrap();

        // Start PRU motors
        controller.start().unwrap();

        // Upon start, PRU motors should send an event
        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .context("Waiting for pru motors status")
            .unwrap();
        for event in events.iter() {
            assert_eq!(PRU_STATUS, event.token());
        }
        assert!(controller.handle_event());
        controller.pru.intc.clear_sysevt(Sysevt::S20); // Rearm interrupt for mock

        println!("Motor started");
        thread::sleep(Duration::from_secs(1));

        for i in 0..4 {
            thread::sleep(Duration::from_secs(1));
            println!("Starting lso want to chmotor {}", i);
            controller.set_motor_speed(i, 240_000).unwrap();
            thread::sleep(Duration::from_secs(3));
            controller.set_motor_speed(i, 199_999).unwrap();
        }

        println!("Stopping motor...");
        // Send stop event
        controller.stop();
        // Wait for acknowledge
        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .context("Waiting for pru motors status")
            .unwrap();
        for event in events.iter() {
            assert_eq!(PRU_STATUS, event.token());
        }
        assert!(!controller.handle_event());
        println!("Motor stopped");
    }
}
