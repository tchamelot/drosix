use drone::sensor::Sensors;

use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

use mio::{unix::SourceFd, Events, Interest, Poll, Token};

const IMU: Token = Token(0);
const STDIN: Token = Token(1);

fn main() {
    let mut sensors = Sensors::new().unwrap();

    let mut poll = Poll::new().expect("Could not create poll");
    let mut events = Events::with_capacity(32);
    poll.registry()
        .register(
            sensors.register_imu_event().unwrap(),
            IMU,
            Interest::READABLE,
        )
        .expect("Cannot register mpu9250 event in epoll");

    poll.registry()
        .register(
            &mut SourceFd(&std::io::stdin().as_raw_fd()),
            STDIN,
            Interest::READABLE,
        )
        .expect("Cannot register stdin event in epoll");

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    writeln!(&mut stdout, "Type enter to exit").unwrap();
    writeln!(&mut stdout, "  Euler angles").unwrap();

    'main: loop {
        poll.poll(&mut events, Some(Duration::from_millis(1000)))
            .expect("could not poll mpu9250");
        for event in events.iter() {
            match event.token() {
                IMU => {
                    let odo = sensors.handle_imu_event().unwrap();
                    write!(
                        &mut stdout,
                        "\r{:>6.1} {:>6.1} {:>6.1} | {:>2.1} ",
                        odo.euler[0].to_degrees(),
                        odo.euler[1].to_degrees(),
                        odo.euler[2].to_degrees(),
                        odo.thrust
                    )
                    .unwrap();
                    stdout.flush().unwrap();
                },
                STDIN => break 'main,
                _ => write!(&mut stdout, "\nUnknown event\n").unwrap(),
            }
        }
        if events.is_empty() {
            write!(&mut stdout, "\nTimeout\n").unwrap();
            sensors.clean_imu().unwrap();
        }
    }
}
