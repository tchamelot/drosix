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
    None,
    PidLoop,
    PidNewData,
    PwmStep,
    PwmChange,
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
#[derive(Clone, Copy, Debug, Default)]
pub struct FlightCommand {
    pub thrust: f32,
    pub angles: Angles,
}

#[derive(Debug)]
pub enum Command {
    Flight(FlightCommand),
    SwitchDebug(DebugConfig),
    Armed(bool),
    SetMotor {
        motor: usize,
        value: u32,
    },
    Stop,
}
