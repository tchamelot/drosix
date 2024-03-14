use crate::config::DrosixParameters;
use crate::controller::PruController;
use crate::polling::Poller;
use crate::sensor::Sensors;
use crate::types::{Command, FlightCommand, Log};

use mio::{Interest, Token};

use std::sync::mpsc::{Receiver, Sender};

use anyhow::{Context, Result};

use prusst::Pruss;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

const LOG_FILE: &'static str = "/tmp/drosix.log";

const IMU: Token = Token(0);
const CONTROLLER: Token = Token(1);
const DEBUG: Token = Token(2);

pub struct FlightController {
    last_cmd: Option<FlightCommand>,
    server_rx: Receiver<Command>,
    server_tx: Sender<Log>,
}

impl<'a> FlightController {
    pub fn new(server_rx: Receiver<Command>, server_tx: Sender<Log>) -> Self {
        Self {
            last_cmd: None,
            server_rx,
            server_tx,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let config = DrosixParameters::load()?;
        let mut poller = Poller::new(8)?;

        let mut pru = Pruss::new(&PruController::config()).context("Instanciating PRUSS")?;
        let mut controller = PruController::new(&pru.intc, &mut pru.dram2);
        poller.register(&controller.status, CONTROLLER, Interest::READABLE)?;
        poller.register(&controller.debug, DEBUG, Interest::READABLE)?;

        let mut sensors = Sensors::new()?;

        let mut log = File::create(LOG_FILE).context("Cannot create a log file")?;

        poller.register(&sensors.imu_event(), IMU, Interest::READABLE)?;

        controller.set_rate_pid(config.rate_pid);
        controller.set_rate_pid(config.attitude_pid);
        controller.switch_debug(config.debug_config);
        PruController::start(&mut pru.pru0, &mut pru.pru1)?;

        let start = Instant::now();

        'control_loop: loop {
            let events = poller.poll(Some(Duration::from_millis(20)))?;
            if events.is_empty() {
                // println!("IMU timeout");
                sensors.handle_imu_event()?;
                sensors.clean_imu()?;
            }
            for event in events.iter() {
                match event.token() {
                    IMU => self.fly(&mut sensors, &mut controller)?,
                    CONTROLLER => {
                        if !controller.handle_event() {
                            break 'control_loop;
                        }
                    },
                    DEBUG => {
                        controller.handle_debug();
                        log.write_all(&start.elapsed().as_millis().to_le_bytes()).unwrap();
                        log.write_all(controller.dump_raw()).unwrap();

                        // println!(
                        //     "[{}] {:?}",
                        //     start.elapsed().as_millis(),
                        //     shared_mem.pid_input
                        // );
                    },
                    _ => (),
                }
            }
            self.handle_command(&mut controller);
        }

        Ok(())
    }

    fn fly(&mut self, sensors: &mut Sensors, controller: &mut PruController) -> Result<()> {
        let mut measures = sensors.handle_imu_event()?;

        measures.thrust = 0.0;
        // Keeps for adjusting purpose
        // let mut inputs = [
        //     (-measures.euler[1]) as f32, // p_measure_x
        //     (-measures.euler[0]) as f32, // p_measure_y
        //     (-measures.euler[2]) as f32, // p_measure_z
        //     (0) as f32,                  // thrust
        //     (-measures.gyro[1]) as f32,  // v_measure_x
        //     (-measures.gyro[0]) as f32,  // v_measure_y
        //     (-measures.gyro[2]) as f32,  // v_measure_z
        // ];
        if let Some(command) = self.last_cmd {
            measures.thrust += command.thrust * 99999.0;
        }
        controller.set_pid_inputs(measures);
        Ok(())
    }

    fn handle_command(&mut self, controller: &mut PruController) {
        match self.server_rx.try_recv() {
            Ok(Command::Flight(cmd)) => {
                self.last_cmd = Some(cmd);
            },
            Ok(Command::SwitchDebug(dbg)) => controller.switch_debug(dbg),
            Ok(Command::Armed(true)) => controller.set_armed(),
            Ok(Command::Armed(false)) => controller.clear_armed(),
            _ => {},
        }
    }
}
