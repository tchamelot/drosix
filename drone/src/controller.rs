use anyhow::{bail, Context, Result};
use prusst::util::VolatileCell;
use prusst::{Channel, Evtout, EvtoutIrq, Host, Intc, IntcConfig, MemSegment, PruLoader, Sysevt};

use std::fs::File;

use crate::types::{AnglePid, Angles, DebugConfig, Odometry, Pid};

const MOTORS_FW: &str = "/lib/firmware/motor.bin";
const PID_FW: &str = "/lib/firmware/controller.bin";

/// Shared memory between the Cortex-A8 and the two PRUs.
/// This structure should only be allocated once by the PRU controller.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SharedMem {
    /// PID parameters for attitude controller
    pub attitude_pid: VolatileCell<AnglePid>,
    /// PID parameters for thrust controller
    pub thrust_pid: VolatileCell<Pid>,
    /// PID parameters for rate controller
    pub rate_pid: VolatileCell<AnglePid>,
    /// PID controller inp
    pub pid_input: VolatileCell<Odometry>,
    /// PID controller outputs: Motor[1-4] duty cycles
    pub pid_output: [VolatileCell<u32>; 4],
    /// For debug purpose: position PID outputs
    pub p_pid: VolatileCell<Angles>,
    /// For debug purpose: speed PID outputs
    pub v_pid: VolatileCell<Angles>,
    /// For debug purpose: number of cycles recorded by a PRU
    pub cycle: VolatileCell<u32>,
    /// For debug purpose: number of stall cycles recorded by a PRU
    pub stall: VolatileCell<u32>,
    /// For debug purpose: indicates which event should trigger a debug event
    pub debug_config: VolatileCell<DebugConfig>,
}

impl Default for SharedMem {
    fn default() -> Self {
        SharedMem {
            attitude_pid: VolatileCell::new(AnglePid::default()),
            thrust_pid: VolatileCell::new(Pid::default()),
            rate_pid: VolatileCell::new(AnglePid::default()),
            debug_config: VolatileCell::new(DebugConfig::None),
            pid_input: VolatileCell::new(Odometry::default()),
            pid_output: [VolatileCell::new(179_999); 4],
            p_pid: VolatileCell::new(Angles::default()),
            v_pid: VolatileCell::new(Angles::default()),
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

/// Abstraction for PRU interface.
pub struct PruController<'a> {
    /// File handle for PRU status interrupt
    pub status: EvtoutIrq,
    /// File handle for PRU debug interrupt
    pub debug: EvtoutIrq,
    intc: &'a Intc,
    running: bool,
    shared_mem: &'a mut SharedMem,
}

impl<'a> PruController<'a> {
    /// Creates a new controller taking ownership of the PRU component by holding references to them.
    pub fn new(intc: &'a Intc, mem: &'a mut MemSegment) -> Self {
        let status = intc.register_irq(Evtout::E0);
        let debug = intc.register_irq(Evtout::E1);
        let shared_mem = mem.alloc(SharedMem::default());
        Self {
            status,
            debug,
            intc,
            shared_mem,
            running: false,
        }
    }

    /// Return expected interrupt config for this controller.
    pub fn config() -> IntcConfig {
        let mut int_conf = IntcConfig::new_empty();
        int_conf.map_sysevts_to_channels(&EVENT_MAP);
        int_conf.map_channels_to_hosts(&CHANNEL_MAP);
        int_conf.auto_enable_sysevts();
        int_conf.auto_enable_hosts();
        int_conf
    }

    pub fn set_attitude_pid(&mut self, config: AnglePid) {
        self.shared_mem.attitude_pid.set(config);
    }

    pub fn set_rate_pid(&mut self, config: AnglePid) {
        self.shared_mem.rate_pid.set(config);
    }

    pub fn set_thrust_pid(&mut self, config: Pid) {
        self.shared_mem.thrust_pid.set(config);
    }

    /// Starts the PRU (load and launch firmwares).
    pub fn start(pru0: &mut PruLoader, pru1: &mut PruLoader) -> Result<()> {
        // Load PRU code
        let mut pid_fw = File::open(PID_FW).context("Opening PID controller firmware")?;
        let mut motor_fw = File::open(MOTORS_FW).context("Opening ESC controller firmware")?;
        let mut contoller_code = pru0.load_code(&mut pid_fw).context("Loading PID controller firmware")?;
        let mut motor_code = pru1.load_code(&mut motor_fw).context("Loading ESC controller firmware")?;
        // TODO get handle over PruCode
        unsafe {
            contoller_code.run();
            motor_code.run();
        }
        Ok(())
    }

    /// Handles a status event
    /// Return true if the PRUs are running
    /// Return false if the PRUs have stopped whether because of an error or because of the natural ending of the
    /// firmware
    ///
    /// This function shall be called to re-enable status event.
    pub fn handle_event(&mut self) -> bool {
        self.intc.clear_sysevt(Sysevt::S19);
        self.intc.enable_host(Evtout::E0);
        if !self.running {
            self.running = true;
        } else {
            self.running = false;
        }

        self.running
    }

    /// Handles a debug event
    ///
    /// This function shall be called to re-enable debug event.
    pub fn handle_debug(&mut self) -> &SharedMem {
        self.intc.clear_sysevt(Sysevt::S31);
        self.intc.enable_host(Evtout::E1);
        self.shared_mem
    }

    /// Set speed for the given motor
    /// The speed shall be between 199999 and 299999.
    ///
    /// This function will return an error for other speed values.
    pub fn set_motor_speed(&mut self, motor: usize, speed: u32) -> Result<()> {
        if motor > 3 {
            bail!("Cannot set speed for motor {}", motor);
        }
        if speed < 199_999 || speed > 399_999 {
            bail!("Cannot set motor {} speed to {} range is [199999;399999]", motor, speed);
        }
        self.shared_mem.pid_output[motor].set(speed);
        self.intc.send_sysevt(Sysevt::S21);
        Ok(())
    }

    /// Sends new values to the PID controller
    /// New values will be processed only if the motor are [armed](PruController::set_armed).
    pub fn set_pid_inputs(&mut self, inputs: Odometry) {
        self.shared_mem.pid_input.set(inputs);
        self.intc.send_sysevt(Sysevt::S18);
    }

    /// Arms the motor making the PID controller start.
    pub fn set_armed(&mut self) {
        self.intc.send_sysevt(Sysevt::S22);
    }

    /// Disarms the motor making the PID controller stop.
    pub fn clear_armed(&mut self) {
        self.intc.send_sysevt(Sysevt::S23);
    }

    /// Changes the PRU debug configuration
    /// If the configuration is different from [`DebugConfig::None`] the PRUs will trigger an event
    /// through [`PruController::debug`]. Use [`PruController::handle_debug`] after a debug event to
    /// re-enable the debug event.
    pub fn switch_debug(&mut self, dbg: DebugConfig) {
        self.shared_mem.debug_config.set(dbg);
    }

    /// Stops the PRU subsystems.
    /// The stop is effective after receiving a new status change event.
    /// The [`PruController::handle_event`] function should return false after this event.
    pub fn stop(&mut self) {
        if self.running {
            self.intc.send_sysevt(Sysevt::S16);
        }
    }

    /// Dump the content of the shared memory related to the PID to a bytearray
    /// Read the code to see the bytearray layout.
    pub fn dump_raw(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (&self.shared_mem.p_pid as *const VolatileCell<Angles>) as *const u8,
                std::mem::size_of::<VolatileCell<Angles>>() + std::mem::size_of::<VolatileCell<Angles>>(),
            )
        }
    }

    pub fn read_pid(&self) -> (Angles, Angles) {
        (self.shared_mem.p_pid.get(), self.shared_mem.v_pid.get())
    }
}

impl<'a> Drop for PruController<'a> {
    fn drop(&mut self) {
        self.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polling::Poller;
    use mio::{Interest, Token};
    use prusst::Pruss;
    use std::time::Duration;

    #[test]
    fn test_controller() {
        // Setup interrupt infrastructure
        let mut pru = Pruss::new(&PruController::config()).context("Instanciating PRUSS").unwrap();
        let mut poller = Poller::new(8).unwrap();
        const PRU_STATUS: Token = Token(0);
        const PRU_DEBUG: Token = Token(1);

        let mut controller = PruController::new(&pru.intc, &mut pru.dram2);
        let pids = AnglePid {
            roll: Pid {
                numerator: [10.0, 0.0, 0.0],
                denominator: [0.0, 0.0],
            },
            pitch: Pid {
                numerator: [10.0, 0.0, 0.0],
                denominator: [0.0, 0.0],
            },
            yaw: Pid {
                numerator: [10.0, 0.0, 0.0],
                denominator: [0.0, 0.0],
            },
        };
        controller.set_attitude_pid(pids);
        controller.set_rate_pid(pids);

        poller.register(&controller.status, PRU_STATUS, Interest::READABLE).unwrap();
        poller.register(&controller.debug, PRU_DEBUG, Interest::READABLE).unwrap();

        // Start sequence
        PruController::start(&mut pru.pru0, &mut pru.pru1).context("Cannot start thre PRUs").unwrap();
        let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
        if events.is_empty() {
            panic!("PRUs did not start correctly");
        } else {
            let mut event_counter = 0;
            for event in events.iter() {
                if event.token() != PRU_STATUS {
                    panic!("PRUs sent an unexpected interrupt at start-up");
                }
                event_counter += 1;
            }
            if event_counter != 1 {
                panic!("PRUs sent too many interrupt at start-up");
            }
        }
        assert!(controller.handle_event());

        // Check start unarmed
        controller.switch_debug(DebugConfig::PwmChange);
        let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
        if !events.is_empty() {
            panic!("PRUs sent too many event_counter while unarmed");
        }

        // Check arming
        controller.set_armed();
        let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
        if events.is_empty() {
            panic!("PRUs did not armed correctly");
        } else {
            for event in events.iter() {
                if event.token() != PRU_DEBUG {
                    panic!("PRUs sent an unexpected interrupt while armed");
                }
            }
        }
        controller.handle_debug();

        // Check sending data
        controller.switch_debug(DebugConfig::PidLoop);
        let input = Odometry {
            attitude: Angles {
                roll: 1.1,
                pitch: 2.2,
                yaw: 3.2,
            },
            rate: Angles {
                roll: 4.4,
                pitch: 5.5,
                yaw: 6.6,
            },
            thrust: 7.7,
        };
        for _ in 0..10 {
            controller.set_pid_inputs(input);
            let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
            if events.is_empty() {
                panic!("PRUs did not receive new data");
            } else {
                for event in events.iter() {
                    if event.token() != PRU_DEBUG {
                        panic!("PRUs sent an unexpected interrupt while armed");
                    }
                }
            }
            let shmem = controller.handle_debug();
            assert_eq!(shmem.pid_input.get(), input);
        }
        // TODO assert
        println!("{:#x?}", controller.shared_mem.p_pid.get());
        println!("{:#x?}", controller.shared_mem.v_pid.get());

        // Check unarming
        controller.switch_debug(DebugConfig::PidLoop);
        controller.clear_armed();
        let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
        if !events.is_empty() {
            panic!("PRUs sent too many event_counter while unarmed");
        }
        controller.handle_debug();

        // Stop sequence
        controller.stop();
        let events = poller.poll(Some(Duration::from_secs(1))).unwrap();
        if events.is_empty() {
            panic!("PRUs did not stop correctly");
        }
        assert!(!controller.handle_event());
    }
}
