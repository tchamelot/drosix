use anyhow::{Context, Result};
use config::Config;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::types::{AnglePid, DebugConfig};

const CONFIG_FILE: &'static str = "drosix.toml";

pub static DROSIX_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::builder().add_source(config::File::with_name(CONFIG_FILE)).build().expect("Loading Drosix config")
});

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
        let config = std::fs::read_to_string(CONFIG_FILE).context("Cannot open configuration file")?;
        toml::from_str(&config).context("Cannot parse configuration file")
    }

    pub fn update(&self) -> Result<()> {
        let config = toml::to_string_pretty(self)?;
        std::fs::write(CONFIG_FILE, config).context("Cannot write configuration file")
    }
}
