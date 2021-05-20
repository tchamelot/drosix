use drone::controller::PruController;
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
use std::os::unix::io::AsRawFd;

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
        .register(
            &mut SourceFd(&std::io::stdin().as_raw_fd()),
            STDIN,
            Interest::READABLE,
        )
        .expect("Cannot register stdin event in epoll");

    controller.start().unwrap();
    println!("Power [%] (0.0, 1.0) or -1 to stop: ");
    'main: loop {
        poll.poll(&mut events, None).expect("could not poll event");
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
                    controller.handle_debug();
                },
                STDIN => {
                    let mut val = String::new();
                    std::io::stdin()
                        .read_line(&mut val)
                        .expect("failed to read input");
                    println!("Starting test");
                    controller.set_pid_inputs([0, 0, 0, 0]);
                },
                _ => println!("Unexpected event polled"),
            }
        }
    }

    println!("Goodbye!");
}
