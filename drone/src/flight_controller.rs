use crate::config::DrosixParameters;
use crate::controller::PruController;
use crate::polling::Poller;
use crate::sensor::{Error, Sensors};
use crate::types::{Command, FlightCommand, Pid};

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
    server_tx: Sender<()>,
}

impl<'a> FlightController {
    pub fn new(server_rx: Receiver<Command>, server_tx: Sender<()>) -> Self {
        Self {
            last_cmd: None,
            server_rx,
            server_tx,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Started flight controller");
        let config = DrosixParameters::load()?;
        let mut poller = Poller::new(8)?;

        let mut pru = Pruss::new(&PruController::config()).context("Instanciating PRUSS")?;
        let mut controller = PruController::new(&pru.intc, &mut pru.dram2);
        poller.register(&controller.status, CONTROLLER, Interest::READABLE)?;
        poller.register(&controller.debug, DEBUG, Interest::READABLE)?;

        let mut sensors = Sensors::new()?;

        poller.register(&sensors.imu_event(), IMU, Interest::READABLE)?;

        controller.set_rate_pid(config.rate_pid);
        controller.set_attitude_pid(config.attitude_pid);
        controller.set_thrust_pid(Pid {
            numerator: [1.0, 0.0, 0.0],
            denominator: [0.0, 0.0],
        });
        controller.switch_debug(config.debug_config);

        PruController::start(&mut pru.pru0, &mut pru.pru1)?;
        let start = Instant::now();

        'control_loop: loop {
            let events = poller.poll(Some(Duration::from_millis(20)))?;
            if events.is_empty() {
                // println!("IMU timeout");
                log::warn!("IMU event timed out");
                sensors.handle_imu_event()?;
                sensors.clean_imu()?;
            }
            for event in events.iter() {
                match event.token() {
                    IMU => {
                        self.fly(&mut sensors, &mut controller).or_else(|err| match err.downcast_ref::<Error>() {
                            // TODO Handle error as critical if the drone is armed or flying
                            Some(Error::NotCalibarated) => Ok(()),
                            Some(Error::NotAvailable) => Ok(log::warn!("IMU data not available")),
                            _ => Err(err),
                        })?
                    },
                    CONTROLLER => {
                        if !controller.handle_event() {
                            log::info!("Flight controller stopped");
                            break 'control_loop;
                        }
                    },
                    DEBUG => {
                        controller.handle_debug();
                        // log.write_all(&start.elapsed().as_millis().to_le_bytes()).unwrap();
                        // log.write_all(controller.dump_raw()).unwrap();
                        log::debug!("{}", unsafe { std::str::from_utf8_unchecked(controller.dump_raw()) });
                    },
                    _ => (),
                }
            }
            self.handle_command(&mut controller);
        }

        Ok(())
    }

    #[cfg_attr(feature = "profiling", function_timer::time("drosix"))]
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
        measures.attitude.roll *= -1.0;
        measures.attitude.pitch *= -1.0;
        measures.attitude.yaw *= -1.0;
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
            Ok(Command::SwitchDebug(dbg)) => {
                log::info!("Switching debug mode to {:?}", dbg);
                controller.switch_debug(dbg);
            },
            Ok(Command::Armed(true)) => {
                log::info!("Arming");
                controller.set_armed();
            },
            Ok(Command::Armed(false)) => {
                log::info!("Disarming");
                controller.clear_armed();
            },
            Ok(Command::SetMotor {
                motor,
                value,
            }) => controller.set_motor_speed(motor, value).unwrap_or_else(|e| log::warn!("{}", e)),
            Ok(Command::Stop) => {
                log::warn!("Stoping flight controller");
                controller.stop();
            },
            Ok(other) => log::warn!("Command not handled: {:?}", other),
            _ => {},
        }
    }
}
