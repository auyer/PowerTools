use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Limits {
    pub cpu: CpuLimit,
    pub gpu: GpuLimit,
    pub battery: BatteryLimit,
}

impl Limits {
    pub fn apply_override(&mut self, limit_override: Option<Self>) {
        if let Some(limit_override) = limit_override {
            self.cpu.limits.apply_override(limit_override.cpu.limits);
            self.gpu.limits.apply_override(limit_override.gpu.limits);
            self.battery.limits.apply_override(limit_override.battery.limits);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Limit<P, L> {
    pub provider: P,
    pub limits: L,
}

pub type CpuLimit = Limit<super::CpuLimitType, super::GenericCpusLimit>;
pub type GpuLimit = Limit<super::GpuLimitType, super::GenericGpuLimit>;
pub type BatteryLimit = Limit<super::BatteryLimitType, super::GenericBatteryLimit>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LimitExtras {
    pub experiments: bool,
    pub quirks: std::collections::HashSet<String>,
}
