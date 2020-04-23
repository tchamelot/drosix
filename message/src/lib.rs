use anyhow::Error;

pub use speedy::Readable;
pub use speedy::Writable;

#[derive(Readable, Writable, Debug)]
pub enum DrosixMessage {
    ClientHello,
    ServerHello(u32),
    Measure([f64; 3]),
    Control([f64; 3]),
    Error,
}

impl From<Result<Vec<u8>, Error>> for DrosixMessage {
    fn from(msg: Result<Vec<u8>, Error>) -> Self {
        msg.and_then(|msg: Vec<u8>| DrosixMessage::read_from_buffer(&msg).map_err(Error::new))
            .unwrap_or(DrosixMessage::Error)
    }
}

impl Into<Result<Vec<u8>, Error>> for DrosixMessage {
    fn into(self) -> Result<Vec<u8>, Error> {
        self.write_to_vec().map_err(Error::new)
    }
}

// fn main() {
//     let hello = DrosixMessage::ClientHello;
//     let serialized = hello.write_to_vec();
//     dbg!(&serialized);
//     dbg!(DrosixMessage::read_from_buffer(
//         serialized.unwrap().as_slice()
//     ));
//     let hello = DrosixMessage::ServerHello(123);
//     let serialized = hello.write_to_vec();
//     dbg!(&serialized);
//     dbg!(DrosixMessage::read_from_buffer(
//         serialized.unwrap().as_slice()
//     ));
//     let measure = DrosixMessage::Measure([0.0, 0.1, 0.2, 0.3]);
//     let serialized = measure.write_to_vec();
//     dbg!(&serialized);
//     dbg!(DrosixMessage::read_from_buffer(
//         serialized.unwrap().as_slice()
//     ));
//     let control = DrosixMessage::Control([0.0, 1.0, 2.0, 3.0]);
//     let serialized = control.write_to_vec();
//     dbg!(&serialized);
//     dbg!(DrosixMessage::read_from_buffer(
//         serialized.unwrap().as_slice()
//     ));
// }
