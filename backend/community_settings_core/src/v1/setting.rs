use serde::{Deserialize, Serialize};

/// Base setting file containing all information for all components
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub cpus: Vec<Cpu>,
    pub gpu: Gpu,
    pub battery: Battery,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct MinMax<T> {
    pub max: Option<T>,
    pub min: Option<T>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Battery {
    pub charge_rate: Option<u64>,
    pub charge_mode: Option<String>,
    #[serde(default)]
    pub events: Vec<BatteryEvent>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BatteryEvent {
    pub trigger: String,
    pub charge_rate: Option<u64>,
    pub charge_mode: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cpu {
    pub online: bool,
    pub clock_limits: Option<MinMax<u64>>,
    pub governor: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Gpu {
    pub fast_ppt: Option<u64>,
    pub slow_ppt: Option<u64>,
    pub tdp: Option<u64>,
    pub tdp_boost: Option<u64>,
    pub clock_limits: Option<MinMax<u64>>,
    pub slow_memory: bool,
}
