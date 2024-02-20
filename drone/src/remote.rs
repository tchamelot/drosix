use crate::config::DebugConfig;
use crate::messages::Command;
use gilrs::{Button, Event, EventType, Gilrs};
use std::thread;
use std::time;
use tokio::sync::mpsc::Sender;

pub fn remote(remote_tx: Sender<Command>) {
    let mut gilrs = Gilrs::new().unwrap();

    'main: loop {
        while let Some(Event {
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
                    remote_tx
                        .blocking_send(Command::Flight([value.into(), 0.0, 0.0, 0.0]))
                        .expect("Cannot send command from remote to drone");
                },
                EventType::ButtonPressed(Button::DPadLeft, _) => {
                    remote_tx
                        .blocking_send(Command::SubscribeDebug(DebugConfig::PidLoop))
                        .expect("Cannot send debug command from remote to drone");
                },
                _ => {
                    // println!("Not handled event: {:?}", event);
                },
            }
        }
        std::thread::sleep(time::Duration::from_millis(50));
    }
}
