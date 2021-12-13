use crate::controller::Pid;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum Command {
    ClientHello,
    Flight([f64; 4]),
    SetPid {
        pid: usize,
        config: Pid,
    },
    CommitPid,
    GetPid(usize),
    SubscribeDebug(u32),
    UnsubscribeDebug(u32),
    Arm,
    Disarm,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum Answer {
    ServerHello(u32),
    Pid {
        pid: usize,
        config: Pid,
    },
    Debug {
        pid_input: [f32; 7],
        pid_output: [f32; 4],
        debug_location: u32,
        p_pid: [f32; 3],
        v_pid: [f32; 3],
        cycle: u32,
        stall: u32,
    },
    Error,
}

// #[derive(Readable, Writable, Debug)]
// pub enum DrosixMessage {
//     ClientHello,
//     ServerHello(u32),
//     Measure([f32; 3]),
//     Debug {
//         pid_input: [f32; 7],
//         pid_output: [f32; 4],
//         debug_location: u32,
//         p_pid: [f32; 3],
//         v_pid: [f32; 3],
//         cycle: u32,
//         stall: u32,
//     },
//     Control([f64; 4]),
//     Error,
// }
//
// impl From<Result<Vec<u8>, Error>> for DrosixMessage {
//     fn from(msg: Result<Vec<u8>, Error>) -> Self {
//         msg.and_then(|msg: Vec<u8>| {
//             DrosixMessage::read_from_buffer(&msg).map_err(Error::new)
//         })
//         .unwrap_or(DrosixMessage::Error)
//     }
// }
