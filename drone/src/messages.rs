use crate::config::{AnglePid, DebugConfig, Pid};

pub enum Command {
    ClientHello,
    Flight([f64; 4]),
    SetPid {
        pid: usize,
        config: Pid,
    },
    CommitPid,
    GetPid(usize),
    SubscribeDebug(DebugConfig),
    UnsubscribeDebug(DebugConfig),
    Arm,
    Disarm,
}

pub enum Answer {
    ServerHello(u32),
    Pid {
        pid: usize,
        config: Pid,
    },
    Debug {
        pid_input: [f32; 7],
        pid_output: [f32; 4],
        debug_location: DebugConfig,
        p_pid: [f32; 3],
        v_pid: [f32; 3],
        cycle: u32,
        stall: u32,
    },
    Error,
}
