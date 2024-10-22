use serde::{Deserialize, Serialize};

/// Proportional Integral Derivative controller parameters
#[repr(C)]
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug)]
pub struct PidConfig {
    /// Proportional gain for attitude
    pub kpa: f32,
    /// Proportional gain for rate
    pub kpr: f32,
    /// Integral constant
    pub ti: f32,
    /// Derivative constant
    pub td: f32,
    /// Derivative filter
    pub filter: f32,
    /// Anti windup factor
    pub kaw: f32,
    /// Upper limit
    pub max: f32,
    /// Lower limit
    pub min: f32,
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
