use gilrs::{Button, Event, EventType, Gilrs};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use crate::types::{Angles, Command, DebugConfig, FlightCommand};

const MOTOR_OFF: u32 = 199_999;
const MOTOR_ON: u32 = 215_000;

pub fn remote(remote_tx: Sender<Command>) {
    let mut gilrs = Gilrs::new().unwrap();
    let mut armed = false;
    let mut watchdog = Instant::now();
    let mut motor_on = MOTOR_ON;

    'main: loop {
        if let Some(Event {
            id,
            event,
            time,
        }) = gilrs.next_event()
        {
            match event {
                EventType::Disconnected => {
                    log::info!("New event from {}: Disconected", id);
                    break 'main;
                },
                EventType::Connected => {
                    log::info!("New event from {}: Conected", id);
                },
                EventType::ButtonChanged(Button::LeftTrigger2, value, _) => {
                    watchdog = Instant::now();
                    // First command to take off so the motors shall start
                    if !armed && value != 0.0 {
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
                EventType::ButtonChanged(
                    button @ Button::North | button @ Button::South | button @ Button::East | button @ Button::West,
                    value,
                    _,
                ) => {
                    let motor = match button {
                        Button::North => 0,
                        Button::East => 1,
                        Button::South => 2,
                        Button::West => 3,
                        _ => unreachable!(),
                    };
                    let value = if value < 0.5 {
                        MOTOR_OFF
                    } else {
                        motor_on
                    };
                    remote_tx
                        .send(Command::SetMotor {
                            motor,
                            value,
                        })
                        .expect("Cannot send debug command from remote to drone");
                },
                EventType::ButtonPressed(Button::DPadUp, _) => {
                    motor_on += 5000;
                    if motor_on > 2 * MOTOR_OFF {
                        motor_on = 2 * MOTOR_OFF;
                    }
                    log::info!("PWM: {} ({}%)", motor_on, (f64::from(motor_on) - 199_999.0) / 1999.99);
                },
                EventType::ButtonPressed(Button::DPadDown, _) => {
                    motor_on -= 5000;
                    if motor_on < MOTOR_OFF {
                        motor_on = MOTOR_OFF;
                    }
                    log::info!("PWM: {} ({}%)", motor_on, (f64::from(motor_on) - 199_999.0) / 1999.99);
                },
                _ => {
                    // println!("Not handled event: {:?}", event);
                },
            }
        } else {
            // No event during the previous second so the motors shall stop
            if armed && watchdog.elapsed().as_secs() > 1 {
                remote_tx.send(Command::Armed(false)).expect("Cannot send disarmed from remote to drone");
                armed = false;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
