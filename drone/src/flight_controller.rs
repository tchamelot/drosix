use crate::controller::{Pid, PruController};
use crate::sensor::Sensors;

use mio::{Events, Interest, Poll, Token};

use std::sync::mpsc::Receiver;
use tokio::sync::broadcast::Sender;

use anyhow::{Context, Result};

use std::time::Duration;

const IMU: Token = Token(0);
const CONTROLLER: Token = Token(1);

const PID_CONF: [Pid; 7] = [
    Pid {
        kp: 0,
        ki: 0,
        kd: 0,
    }, // Roll
    Pid {
        kp: 0,
        ki: 0,
        kd: 0,
    }, // Pitch
    Pid {
        kp: 0,
        ki: 0,
        kd: 0,
    }, // Yaw
    Pid {
        kp: 0,
        ki: 0,
        kd: 0,
    }, // Thrust
    Pid {
        kp: 0,
        ki: 0,
        kd: 0,
    }, // Roll'
    Pid {
        kp: 0,
        ki: 0,
        kd: 0,
    }, // Pitch'
    Pid {
        kp: 0,
        ki: 0,
        kd: 0,
    },
]; // Yaw'

pub struct FlightController<'a> {
    sensors: Sensors,
    controller: PruController<'a>,
    server_rx: Receiver<[f64; 4]>,
    server_tx: Sender<[f64; 3]>,
}

impl<'a> FlightController<'a> {
    pub fn new(
        server_rx: Receiver<[f64; 4]>,
        server_tx: Sender<[f64; 3]>,
    ) -> Result<Self> {
        let sensors = Sensors::new()?;
        let controller = PruController::new()?;
        Ok(Self {
            sensors,
            controller,
            server_rx,
            server_tx,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut poll = Poll::new().context("Creating event poller")?;
        let mut events = Events::with_capacity(8);

        poll.registry()
            .register(
                self.controller.register_pru_evt(),
                CONTROLLER,
                Interest::READABLE,
            )
            .context("Regitering controller event")?;
        poll.registry()
            .register(
                self.sensors.register_imu_event()?,
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
        }

        Ok(())
    }

    fn fly(&mut self) -> Result<()> {
        let measures = self.sensors.handle_imu_event()?;

        if let Some(command) = self.server_rx.try_iter().last() {
            // For the moment no feedback / closed loop
            let error = [
                (command[0] * 1000.0) as i32,
                (command[1]) as i32,
                (command[2]) as i32,
                (command[3]) as i32,
            ];
            self.controller.set_pid_inputs(error);
        }

        let _ = self.server_tx.send(measures.euler);

        Ok(())
    }
}
