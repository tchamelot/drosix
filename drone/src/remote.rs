use gilrs::{Button, Event, EventType, Gilrs};

fn main() {
    let mut gilrs = Gilrs::new().unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    'main: loop {
        // Examine new events
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
                EventType::ButtonChanged(Button::LeftTrigger2, value, _) => {
                    println!("Thrust : {}", value * 100.0);
                },
                _ => (),
            }
            active_gamepad = Some(id);
        }
    }
}
