use std::default::Default;

use serde::{Deserialize, Serialize};

use super::{BatteryJson, CpuJson, DriverJson, GpuJson};

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingsJson {
    pub version: u64,
    pub name: String,
    pub variant: u64,
    pub persistent: bool,
    pub cpus: Vec<CpuJson>,
    pub gpu: GpuJson,
    pub battery: BatteryJson,
    pub provider: Option<DriverJson>,
}

impl Default for SettingsJson {
    fn default() -> Self {
        Self {
            version: 0,
            name: crate::consts::DEFAULT_SETTINGS_VARIANT_NAME.to_owned(),
            variant: 0,
            persistent: false,
            cpus: Vec::with_capacity(8),
            gpu: GpuJson::default(),
            battery: BatteryJson::default(),
            provider: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MinMaxJson<T> {
    pub max: Option<T>,
    pub min: Option<T>,
}
