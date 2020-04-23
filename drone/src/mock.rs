use std::{thread, time::Duration};
use tokio::sync::broadcast::Sender;

pub fn drone(sender: Sender<[f64; 3]>) {
    let mut counter: f64 = 0.0;
    loop {
        thread::sleep(Duration::from_millis(20));
        counter += 0.1;
        if counter > 3.14 {
            counter = -3.14;
        }
        let sin = counter.sin();
        let cos = counter.cos();
        let mix = sin * cos;
        sender.send([cos, sin, mix]);
    }
}
