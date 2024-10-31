use crate::config::DROSIX_CONFIG;
use crate::controller::PruController;
use crate::log::{scope, MeasureRecord};
use crate::polling::Poller;
use crate::sensor::{Error, Sensors};
use crate::types::{Command, FlightCommand, Odometry, PidConfig};

use mio::{Interest, Token};

use std::sync::mpsc::{Receiver, Sender};

use anyhow::{Context, Result};

use prusst::Pruss;
use std::time::Duration;

const IMU: Token = Token(0);
const CONTROLLER: Token = Token(1);
const DEBUG: Token = Token(2);

pub struct FlightController {
    command: FlightCommand,
    measures: Odometry,
    server_rx: Receiver<Command>,
    server_tx: Sender<()>,
}

impl<'a> FlightController {
    pub fn new(server_rx: Receiver<Command>, server_tx: Sender<()>) -> Self {
        Self {
            command: FlightCommand::default(),
            measures: Odometry::default(),
            server_rx,
            server_tx,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Started flight controller");
        let mut poller = Poller::new(8)?;

        let mut pru = Pruss::new(&PruController::config()).context("Instanciating PRUSS")?;
        let mut controller = PruController::new(&pru.intc, &mut pru.dram2);
        poller.register(&controller.status, CONTROLLER, Interest::READABLE)?;
        poller.register(&controller.debug, DEBUG, Interest::READABLE)?;

        let mut sensors = Sensors::new()?;

        poller.register(&sensors.imu_event(), IMU, Interest::READABLE)?;

        controller.set_pid(
            DROSIX_CONFIG.get("roll_pid")?,
            DROSIX_CONFIG.get("pitch_pid")?,
            DROSIX_CONFIG.get("yaw_pid")?,
            PidConfig {
                kpa: 1.0,
                kpr: 1.0,
                max: 99999.0,
                min: 0.0,
                ..Default::default()
            },
        );
        controller.switch_debug(DROSIX_CONFIG.get("debug_config")?);

        PruController::start(&mut pru.pru0, &mut pru.pru1)?;

        'control_loop: loop {
            let events = poller.poll(Some(Duration::from_millis(20)))?;
            if events.is_empty() {
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
                        let (position_pid, velocity_pid) = controller.read_pid();
                        scope(MeasureRecord {
                            command: self.command,
                            sensor: self.measures,
                            position_pid,
                            velocity_pid,
                        });
                        #[cfg(feature = "profiling")]
                        metrics::histogram!("drosix", "function" => "PRU pid")
                            .record(controller.read_cycle() as f64 / 200e6);
                    },
                    _ => (),
                }
            }
            self.handle_command(&mut controller);
        }

        Ok(())
    }

    // TODO rerun profiling
    #[cfg_attr(feature = "profiling", function_timer::time("drosix"))]
    fn fly(&mut self, sensors: &mut Sensors, controller: &mut PruController) -> Result<()> {
        let mut measures = sensors.handle_imu_event()?;
        self.measures = measures;

        measures.thrust = 0.0;
        measures.thrust += self.command.thrust * 99999.0;
        measures.attitude.roll *= -1.0;
        measures.attitude.roll += self.command.angles.roll * f32::to_radians(15.0);
        measures.attitude.pitch *= -1.0;
        measures.attitude.pitch += self.command.angles.pitch * f32::to_radians(15.0);
        measures.attitude.yaw *= -1.0;

        controller.set_pid_inputs(measures);
        Ok(())
    }

    fn handle_command(&mut self, controller: &mut PruController) {
        match self.server_rx.try_recv() {
            Ok(Command::Flight(command)) => {
                self.command = command;
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
                self.command = FlightCommand::default();
            },
            Ok(Command::SetMotor {
                motor,
                value,
            }) => controller.set_motor_speed(motor, value).unwrap_or_else(|e| log::warn!("{}", e)),
            Ok(Command::Stop) => {
                log::warn!("Stoping flight controller");
                controller.stop();
            },
            _ => {},
        }
    }
}
