use prusst::util::VolatileCell;
use prusst::{Channel, Evtout, EvtoutIrq, Host, IntcConfig, Pruss, Sysevt};

use std::fs::File;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Pid {
    pub kp: i32,
    pub ki: i32,
    pub kd: i32,
}

impl Default for Pid {
    fn default() -> Self {
        Pid { kp: 0,
              ki: 0,
              kd: 0 }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Controller {
    pub input: [VolatileCell<i32>; 7],
    pub output: [VolatileCell<u32>; 4],
    pub pid: [VolatileCell<Pid>; 7],
    pub cycle: VolatileCell<u32>,
    pub stall: VolatileCell<u32>,
}

impl Default for Controller {
    fn default() -> Self {
        Controller { input: [VolatileCell::new(-20_000); 7],
                     output: [VolatileCell::new(179_999); 4],
                     pid: [VolatileCell::new(Default::default()); 7],
                     cycle: VolatileCell::new(0),
                     stall: VolatileCell::new(0) }
    }
}

const EVENT_MAP: [(Sysevt, Channel); 9] = [(Sysevt::S17, Channel::C0), /* CONTROLLER_STOP */
                                           (Sysevt::S18, Channel::C0), /* PID_NEW_DATA */
                                           (Sysevt::S20, Channel::C0), /* MOTOR_STATUS */
                                           (Sysevt::S16, Channel::C1), /* MOTOR_STOP */
                                           (Sysevt::S21, Channel::C1), /* PID_OUTPUT */
                                           (Sysevt::S22, Channel::C0), /* SET_ARMED */
                                           (Sysevt::S23, Channel::C0), /* CLEAR_ARMED */
                                           (Sysevt::S19, Channel::C2), /* CONTROLLER_STATUS */
                                           (Sysevt::S31, Channel::C3)]; /* DEBUG */
const CHANNEL_MAP: [(Channel, Host); 4] = [(Channel::C0, Host::Pru0),    /* PRU0 */
                                           (Channel::C1, Host::Pru1), /* PRU1 */
                                           (Channel::C2, Host::Evtout0), /* HOST */
                                           (Channel::C3, Host::Evtout1)]; /* HOST_DEBUG */

pub struct PruController<'a> {
    pru: Pruss<'a>,
    shared_mem: &'a mut Controller,
    status_evt: EvtoutIrq,
    debug_evt: EvtoutIrq,
    running: bool,
}

impl<'a> PruController<'a> {
    pub fn new() -> Result<Self, ()> {
        // Init PRU events
        let mut int_conf = IntcConfig::new_empty();
        int_conf.map_sysevts_to_channels(&EVENT_MAP);
        int_conf.map_channels_to_hosts(&CHANNEL_MAP);
        int_conf.auto_enable_sysevts();
        int_conf.auto_enable_hosts();
        let mut pru = match Pruss::new(&int_conf) {
            Ok(p) => p,
            Err(e) => match e {
                prusst::Error::AlreadyInstantiated => {
                    panic!("Pruss: already in use")
                },
                prusst::Error::PermissionDenied => {
                    panic!("Pruss: permission denied ")
                },
                prusst::Error::DeviceNotFound => {
                    panic!("Pruss: char device not found")
                },
                prusst::Error::OtherDeviceError => {
                    panic!("Pruss: unidentified problem occured")
                },
            },
        };

        // Init PRU shared mem
        let controller =
            pru.dram2.alloc(Controller::default()) as *mut Controller;

        let status_evt = pru.intc.register_irq(Evtout::E0);
        let debug_evt = pru.intc.register_irq(Evtout::E1);

        Ok(PruController { pru,
                           shared_mem: unsafe { &mut *(controller.clone()) },
                           status_evt,
                           debug_evt,
                           running: false })
    }

    pub fn start(&mut self) -> Result<(), ()> {
        // Load PRU code
        let mut controller_bin =
            File::open("/lib/firmware/controller.bin").unwrap();
        let mut motor_bin = File::open("/lib/firmware/motor.bin").unwrap();
        let mut contoller_code =
            self.pru.pru0.load_code(&mut controller_bin).unwrap();
        let mut motor_code = self.pru.pru1.load_code(&mut motor_bin).unwrap();
        unsafe {
            contoller_code.run();
            motor_code.run();
        }
        Ok(())
    }

    pub fn register_pru_evt(&mut self) -> &mut EvtoutIrq {
        &mut self.status_evt
    }

    pub fn register_pru_debug(&mut self) -> &mut EvtoutIrq {
        &mut self.debug_evt
    }

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

    pub fn handle_debug(&mut self) {
        self.pru.intc.clear_sysevt(Sysevt::S31);
        self.pru.intc.enable_host(Evtout::E1);
        //dbg!(self.shared_mem.output);
        dbg!(self.shared_mem.output[0].get());
    }

    pub fn set_pid_inputs(&mut self, inputs: [i32; 4]) {
        self.shared_mem.input[0].set(inputs[0]);
        self.shared_mem.input[1].set(inputs[1]);
        self.shared_mem.input[2].set(inputs[2]);
        self.shared_mem.input[3].set(inputs[3]);
        self.pru.intc.send_sysevt(Sysevt::S18);
    }

    pub fn set_armed(&mut self) {
        self.pru.intc.send_sysevt(Sysevt::S22);
    }

    pub fn clear_armed(&mut self) {
        self.pru.intc.send_sysevt(Sysevt::S23);
    }

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
