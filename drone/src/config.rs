use anyhow::{Context, Result};
use bitmask_enum::bitmask;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &'static str = "drosix.toml";

/// Proportional Integral Derivative controller parameters
#[repr(C)]
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug)]
pub struct Pid {
    /// PID input gains
    pub a: [f32; 3],
    /// PID output gains
    pub b: [f32; 2],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug)]
pub struct AnglePid {
    pub roll: Pid,
    pub pitch: Pid,
    pub yaw: Pid,
}

#[bitmask(u32)]
#[derive(Serialize, Deserialize)]
pub enum DebugConfig {
    PidLoop = 0b1,
    PidNewData = 0b10,
    PwmStep = 0b100,
}

/// Drosix configuration parameters
#[derive(Serialize, Deserialize, Debug)]
pub struct DrosixParameters {
    /// PID controllers for the rate [Roll Pitch Yaw]
    pub rate_pid: AnglePid,
    /// PID controllers for the qttitude [Roll Pitch Yaw]
    pub attitude_pid: AnglePid,
    /// Debug configuration for PRU subsystems
    pub debug_config: DebugConfig,
}

impl DrosixParameters {
    pub fn load() -> Result<Self> {
        let mut config = std::fs::read_to_string(CONFIG_FILE).context("Cannot open configuration file")?;
        toml::from_str(&config).context("Cannot parse configuration file")
    }

    pub fn update(&self) -> Result<()> {
        let config = toml::to_string_pretty(self)?;
        std::fs::write(CONFIG_FILE, config).context("Cannot write configuration file")
    }
}
