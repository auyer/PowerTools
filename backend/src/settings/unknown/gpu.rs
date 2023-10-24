use std::convert::Into;

use limits_core::json_v2::GenericGpuLimit;

use crate::persist::GpuJson;
use crate::settings::MinMax;
use crate::settings::{TGpu, ProviderBuilder};
use crate::settings::{OnResume, OnSet, SettingError};

#[derive(Debug, Clone)]
pub struct Gpu {
    slow_memory: bool, // ignored
}

impl Gpu {
    pub fn system_default() -> Self {
        Self { slow_memory: false }
    }
}

impl ProviderBuilder<GpuJson, GenericGpuLimit> for Gpu {
    fn from_json_and_limits(_persistent: GpuJson, _version: u64, _limits: GenericGpuLimit) -> Self {
        Self::system_default()
    }

    fn from_limits(_limits: GenericGpuLimit) -> Self {
        Self::system_default()
    }
}

impl Into<GpuJson> for Gpu {
    #[inline]
    fn into(self) -> GpuJson {
        GpuJson {
            fast_ppt: None,
            slow_ppt: None,
            clock_limits: None,
            slow_memory: false,
            root: None,
        }
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl crate::settings::OnPowerEvent for Gpu {}

impl TGpu for Gpu {
    fn limits(&self) -> crate::api::GpuLimits {
        crate::api::GpuLimits {
            fast_ppt_limits: None,
            slow_ppt_limits: None,
            ppt_step: 1_000_000,
            tdp_limits: None,
            tdp_boost_limits: None,
            tdp_step: 42,
            clock_min_limits: None,
            clock_max_limits: None,
            clock_step: 100,
            memory_control_capable: false,
        }
    }

    fn json(&self) -> crate::persist::GpuJson {
        self.clone().into()
    }

    fn ppt(&mut self, _fast: Option<u64>, _slow: Option<u64>) {}

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        (None, None)
    }

    fn clock_limits(&mut self, _limits: Option<MinMax<u64>>) {}

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        None
    }

    fn slow_memory(&mut self) -> &mut bool {
        &mut self.slow_memory
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Unknown
    }
}
