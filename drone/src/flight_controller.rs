use crate::controller::{Controller, Pid};
use crate::messages::{Answer, Command};
use crate::sensor::Sensors;

use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};

// use std::sync::mpsc::Receiver;
use tokio::sync::mpsc::{Receiver, Sender};

use anyhow::{Context, Result};

use std::time::Duration;

const IMU: Token = Token(0);
const CONTROLLER: Token = Token(1);

const PID_CONF: [Pid; 7] = [
    Pid {
        a: [0.0; 3],
        b: [0.0; 2],
    }, // Roll
    Pid {
        a: [0.0; 3],
        b: [0.0; 2],
    }, // Pitch
    Pid {
        a: [0.0; 3],
        b: [0.0; 2],
    }, // Yaw
    Pid {
        a: [0.0; 3],
        b: [0.0; 2],
    }, // Thrust
    Pid {
        a: [0.0; 3],
        b: [0.0; 2],
    }, // Roll'
    Pid {
        a: [0.0; 3],
        b: [0.0; 2],
    }, // Pitch'
    Pid {
        a: [0.0; 3],
        b: [0.0; 2],
    },
]; // Yaw'

pub struct FlightController<'a> {
    sensors: Sensors,
    controller: Controller<'a>,
    pids: [Pid; 7],
    last_cmd: Option<[f64; 4]>,
    server_rx: Receiver<Command>,
    server_tx: Sender<Answer>,
}

impl<'a> FlightController<'a> {
    pub fn new(
        server_rx: Receiver<Command>,
        server_tx: Sender<Answer>,
    ) -> Result<Self> {
        let sensors = Sensors::new()?;
        let controller = Controller::new()?;
        Ok(Self {
            sensors,
            controller,
            last_cmd: None,
            pids: [Default::default(); 7],
            server_rx,
            server_tx,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut poll = Poll::new().context("Creating event poller")?;
        let mut events = Events::with_capacity(8);

        poll.registry()
            .register(
                &mut SourceFd(&self.controller.register_pru_evt()),
                CONTROLLER,
                Interest::READABLE,
            )
            .context("Regitering controller event")?;
        poll.registry()
            .register(
                &mut SourceFd(&self.sensors.register_imu_event()?),
                IMU,
                Interest::READABLE,
            )
            .context("Registering imu event")?;

        self.controller.set_pid_configs(PID_CONF);
        self.controller.start()?;

        'control_loop: loop {
            poll.poll(&mut events, Some(Duration::from_millis(1000)))
                .context("Polling events")?;
            for event in events.iter() {
                match event.token() {
                    IMU => self.fly()?,
                    CONTROLLER => {
                        if !self.controller.handle_event() {
                            break 'control_loop;
                        } else {
                            self.controller.set_armed();
                        }
                    },
                    _ => (),
                }
            }
            self.handle_command();
        }

        Ok(())
    }

    fn fly(&mut self) -> Result<()> {
        let measures = self.sensors.handle_imu_event()?;

        let mut inputs = [
            (-measures.euler[0]) as f32, // p_measure_x
            (-measures.euler[1]) as f32, // p_measure_y
            (-measures.euler[2]) as f32, // p_measure_z
            (0) as f32,                  // thrust
            (-measures.gyro[0]) as f32,  // v_measure_x
            (-measures.gyro[1]) as f32,  // v_measure_y
            (-measures.gyro[2]) as f32,  // v_measure_z
        ];
        if let Some(command) = self.last_cmd {
            println!("{:?}", command);
            inputs[3] = inputs[3] + (command[0] / 200.0) as f32;
            self.controller.set_pid_inputs(inputs);
        }
        Ok(())
    }

    fn handle_command(&mut self) {
        match self.server_rx.try_recv() {
            Ok(Command::Flight(cmd)) => {
                self.last_cmd = Some(cmd);
            },
            Ok(Command::SetPid {
                pid,
                config,
            }) => {
                self.pids[pid] = config;
                self.server_tx.blocking_send(Answer::Pid {
                    pid,
                    config: self.pids[pid],
                });
            },
            Ok(Command::CommitPid) => {
                self.controller.set_pid_configs(self.pids);
            },
            Ok(Command::GetPid(pid)) => {
                self.server_tx.blocking_send(Answer::Pid {
                    pid,
                    config: self.pids[pid],
                });
            },
            Ok(Command::SubscribeDebug(dbg)) => self.controller.set_debug(dbg),
            Ok(Command::UnsubscribeDebug(dbg)) => {
                self.controller.reset_debug(dbg)
            },
            Ok(Command::Arm) => self.controller.set_armed(),
            Ok(Command::Disarm) => self.controller.clear_armed(),
            _ => {},
        }
    }
}
