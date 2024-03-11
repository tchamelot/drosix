use serde::{Deserialize, Serialize};

/// Proportional Integral Derivative controller parameters
#[repr(C)]
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug)]
pub struct Pid {
    /// PID input gains
    pub numerator: [f32; 3],
    /// PID output gains
    pub denominator: [f32; 2],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug)]
pub struct AnglePid {
    pub roll: Pid,
    pub pitch: Pid,
    pub yaw: Pid,
}

// #[bitmask(u32)]
#[repr(C)]
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum DebugConfig {
    None = 0,
    PidLoop = 0b1,
    PidNewData = 0b10,
    PwmStep = 0b100,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, PartialEq)]
pub struct Angles {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, PartialEq)]
pub struct Odometry {
    pub attitude: Angles,
    pub rate: Angles,
    pub thrust: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlightCommand {
    pub thrust: f32,
    pub angles: Angles,
}

pub enum Command {
    Flight(FlightCommand),
    SwitchDebug(DebugConfig),
    Armed(bool),
}

pub enum Log {
    Debug {
        pid_input: [f32; 7],
        pid_output: [f32; 4],
        debug_config: DebugConfig,
        p_pid: [f32; 3],
        v_pid: [f32; 3],
        cycle: u32,
        stall: u32,
    },
}
