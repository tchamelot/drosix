use rkyv::{Archive, Deserialize, Serialize};

/// PID controller parameters
#[repr(C)]
#[derive(Archive, Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Pid {
    /// PID input gains
    pub a: [f32; 3],
    /// PID output gains
    pub b: [f32; 2],
}

impl Default for Pid {
    fn default() -> Self {
        Pid {
            a: [0.; 3],
            b: [0.; 2],
        }
    }
}

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

#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
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
