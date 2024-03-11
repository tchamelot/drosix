use crate::config::DrosixParameters;
use crate::controller::Controller;
use crate::sensor::Sensors;
use crate::types::{Command, DebugConfig, FlightCommand, Log};

use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};

use std::sync::mpsc::{Receiver, Sender};

use anyhow::{Context, Result};

use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

const LOG_FILE: &'static str = "/tmp/drosix.log";

const IMU: Token = Token(0);
const CONTROLLER: Token = Token(1);
const DEBUG: Token = Token(2);

pub struct FlightController<'a> {
    config: DrosixParameters,
    sensors: Sensors,
    controller: Controller<'a>,
    last_cmd: Option<FlightCommand>,
    server_rx: Receiver<Command>,
    server_tx: Sender<Log>,
}

impl<'a> FlightController<'a> {
    pub fn new(server_rx: Receiver<Command>, server_tx: Sender<Log>) -> Result<Self> {
        let config = DrosixParameters::load()?;
        let sensors = Sensors::new()?;
        let controller = Controller::new()?;
        Ok(Self {
            sensors,
            controller,
            last_cmd: None,
            config,
            server_rx,
            server_tx,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut poll = Poll::new().context("Creating event poller")?;
        let mut events = Events::with_capacity(8);
        let mut log = File::create(LOG_FILE).context("Cannot create a log file")?;

        poll.registry()
            .register(&mut SourceFd(&self.controller.register_pru_evt()), CONTROLLER, Interest::READABLE)
            .context("Registering controller event")?;
        poll.registry()
            .register(&mut SourceFd(&self.sensors.register_imu_event()?), IMU, Interest::READABLE)
            .context("Registering imu event")?;
        poll.registry()
            .register(&mut SourceFd(&self.controller.register_pru_debug()), DEBUG, Interest::READABLE)
            .context("Registering debug event")?;

        self.controller.set_rate_pid(self.config.rate_pid);
        self.controller.set_rate_pid(self.config.attitude_pid);
        self.controller.switch_debug(self.config.debug_config);
        self.controller.start()?;

        let start = Instant::now();

        'control_loop: loop {
            poll.poll(&mut events, Some(Duration::from_millis(20))).context("Polling events")?;
            if events.is_empty() {
                // println!("IMU timeout");
                self.sensors.handle_imu_event()?;
                self.sensors.clean_imu()?;
            }
            for event in events.iter() {
                match event.token() {
                    IMU => self.fly()?,
                    CONTROLLER => {
                        if !self.controller.handle_event() {
                            break 'control_loop;
                        }
                    },
                    DEBUG => {
                        let shared_mem = self.controller.handle_debug();
                        log.write_all(&start.elapsed().as_millis().to_le_bytes()).unwrap();
                        log.write_all(shared_mem.dump_raw()).unwrap();

                        // println!(
                        //     "[{}] {:?}",
                        //     start.elapsed().as_millis(),
                        //     shared_mem.pid_input
                        // );
                    },
                    _ => (),
                }
            }
            self.handle_command();
        }

        Ok(())
    }

    fn fly(&mut self) -> Result<()> {
        let mut measures = self.sensors.handle_imu_event()?;

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
        self.controller.set_pid_inputs(measures);
        Ok(())
    }

    fn handle_command(&mut self) {
        match self.server_rx.try_recv() {
            Ok(Command::Flight(cmd)) => {
                self.last_cmd = Some(cmd);
            },
            Ok(Command::SwitchDebug(dbg)) => self.controller.switch_debug(dbg),
            Ok(Command::Armed(true)) => self.controller.set_armed(),
            Ok(Command::Armed(false)) => self.controller.clear_armed(),
            _ => {},
        }
    }
}
