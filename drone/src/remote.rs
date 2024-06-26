use gilrs::{Button, Event, EventType, Gilrs};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use crate::types::{Angles, Command, DebugConfig, FlightCommand};

const MOTOR_OFF: u32 = 199_999;
const MOTOR_ON: u32 = 399_999;

pub fn remote(remote_tx: Sender<Command>) {
    let mut gilrs = Gilrs::new().unwrap();
    let mut armed = false;
    let mut watchdog = Instant::now();

    'main: loop {
        if let Some(Event {
            id,
            event,
            time,
        }) = gilrs.next_event()
        {
            match event {
                EventType::Disconnected => {
                    println!("{:?} New event from {}: Disconected", time, id);
                    break 'main;
                },
                EventType::Connected => {
                    println!("{:?} New event from {}: Conected", time, id);
                },
                EventType::ButtonChanged(Button::LeftTrigger2, value, _) => {
                    // First command to take off so the motors shall start
                    if !armed && value != 0.0 {
                        watchdog = Instant::now();
                        armed = true;
                        remote_tx.send(Command::Armed(true)).expect("Cannot send armed from remote to drone");
                    }
                    remote_tx
                        .send(Command::Flight(FlightCommand {
                            thrust: value.into(),
                            angles: Angles::default(),
                        }))
                        .expect("Cannot send command from remote to drone");
                },
                EventType::ButtonPressed(Button::DPadLeft, _) => {
                    remote_tx
                        .send(Command::SwitchDebug(DebugConfig::PidLoop))
                        .expect("Cannot send debug command from remote to drone");
                },
                EventType::ButtonPressed(Button::DPadUp, _) => {
                    remote_tx
                        .send(Command::SwitchDebug(DebugConfig::None))
                        .expect("Cannot send debug command from remote to drone");
                },
                EventType::ButtonChanged(Button::North, value, _) => {
                    let value = if value < 0.5 {
                        MOTOR_OFF
                    } else {
                        MOTOR_ON
                    };
                    remote_tx
                        .send(Command::SetMotor {
                            motor: 0,
                            value,
                        })
                        .expect("Cannot send debug command from remote to drone");
                },
                EventType::ButtonChanged(Button::East, value, _) => {
                    let value = if value < 0.5 {
                        MOTOR_OFF
                    } else {
                        MOTOR_ON
                    };
                    remote_tx
                        .send(Command::SetMotor {
                            motor: 1,
                            value,
                        })
                        .expect("Cannot send debug command from remote to drone");
                },
                EventType::ButtonChanged(Button::South, value, _) => {
                    let value = if value < 0.5 {
                        MOTOR_OFF
                    } else {
                        MOTOR_ON
                    };
                    remote_tx
                        .send(Command::SetMotor {
                            motor: 2,
                            value,
                        })
                        .expect("Cannot send debug command from remote to drone");
                },
                EventType::ButtonChanged(Button::West, value, _) => {
                    let value = if value < 0.5 {
                        MOTOR_OFF
                    } else {
                        MOTOR_ON
                    };
                    remote_tx
                        .send(Command::SetMotor {
                            motor: 3,
                            value,
                        })
                        .expect("Cannot send debug command from remote to drone");
                },
                _ => {
                    // println!("Not handled event: {:?}", event);
                },
            }
        } else {
            // No event during the previous second so the motors shall stop
            if armed && watchdog.elapsed().as_secs() > 5 {
                remote_tx.send(Command::Armed(false)).expect("Cannot send disarmed from remote to drone");
                armed = false;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
