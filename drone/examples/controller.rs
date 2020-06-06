use drone::controller::PruController;
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
use std::os::unix::io::AsRawFd;
use std::time::Duration;
use std::time::Instant;

const CONTROLLER: Token = Token(0);
const DEBUG: Token = Token(1);
const STDIN: Token = Token(2);

fn main() {
    let mut controller = PruController::new().unwrap();
    let mut poll = Poll::new().expect("Could not create poll");
    let mut events = Events::with_capacity(32);

    poll.registry()
        .register(controller.register_pru_evt(), CONTROLLER, Interest::READABLE)
        .expect("Cannot register pru0 event in epoll");

    poll.registry()
        .register(controller.register_pru_debug(), DEBUG, Interest::READABLE)
        .expect("Cannot register pru0 event in epoll");

    poll.registry()
        .register(&mut SourceFd(&std::io::stdin().as_raw_fd()),
                  STDIN,
                  Interest::READABLE)
        .expect("Cannot register stdin event in epoll");

    controller.start().unwrap();
    let mut armed = false;
    println!("Power [%] (0.0, 1.0) or -1 to stop: ");
    let mut now = Instant::now();
    'main: loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .expect("could not poll mpu9250");
        for event in events.iter() {
            match event.token() {
                CONTROLLER => {
                    if !controller.handle_event() {
                        println!("pru stoped");
                        break 'main;
                    } else {
                        println!("pru started");
                    }
                },
                DEBUG => {
                    println!("Debug: {}µs", now.elapsed().as_micros());
                    now = Instant::now();
                    controller.handle_debug();
                },
                STDIN => {
                    if let Some(set_point) = get_input(0.0, 1.0, -0.1) {
                        if set_point != -1 {
                            if !armed && set_point > 10000 {
                                controller.set_armed();
                                armed = true;
                                println!("Motor armed");
                            } else if armed && set_point <= 10000 {
                                controller.clear_armed();
                                armed = false;
                                println!("Motor not armed");
                            }
                            println!("new set point: {}", set_point);
                            controller.set_pid_inputs([0, 0, 0, set_point]);
                        } else {
                            // Stop the system
                            controller.stop();
                        }
                    }
                },
                _ => println!("Unexpected event polled"),
            }
        }
    }

    println!("Goodbye!");
}

fn get_input(min: f32, max: f32, stop: f32) -> Option<i32> {
    let mut val = String::new();
    std::io::stdin().read_line(&mut val).expect("failed to read input");

    if let Ok(val) = val.trim().parse::<f32>() {
        if val >= min && val <= max {
            return Some((val * 199_999.0) as i32);
        } else if val == stop {
            return Some(-1);
        } else {
            println!("the input should be between {} and {}", min, max);
            return None;
        }
    } else {
        println!("not a number");
        return None;
    }
}
