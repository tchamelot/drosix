use gilrs::{Button, Event, EventType, Gilrs};
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::types::{Angles, Command, DebugConfig, FlightCommand};

pub fn remote(remote_tx: Sender<Command>) {
    let mut gilrs = Gilrs::new().unwrap();
    let mut armed = false;

    'main: loop {
        if let Some(Event {
            id,
            event,
            time,
        }) = gilrs.next_event_blocking(Some(Duration::from_secs(5)))
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
                    if !armed {
                        remote_tx.send(Command::Armed(true)).expect("Cannot send armed from remote to drone");
                        armed = true;
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
                _ => {
                    // println!("Not handled event: {:?}", event);
                },
            }
        } else {
            // No event during the previous second so the motors shall stop
            if armed {
                remote_tx.send(Command::Armed(false)).expect("Cannot send disarmed from remote to drone");
                armed = false;
            }
        }
    }
}
