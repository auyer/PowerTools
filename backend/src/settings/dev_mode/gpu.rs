use std::convert::Into;

use limits_core::json_v2::GenericGpuLimit;

use crate::persist::GpuJson;
use crate::settings::MinMax;
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{ProviderBuilder, TGpu};

#[derive(Clone)]
pub struct Gpu {
    persist: GpuJson,
    version: u64,
    limits: GenericGpuLimit,
    clock_limits: Option<MinMax<u64>>,
}

impl std::fmt::Debug for Gpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("dev_mode_Gpu")
            //.field("persist", &self.persist)
            .field("version", &self.version)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

impl ProviderBuilder<GpuJson, GenericGpuLimit> for Gpu {
    fn from_json_and_limits(persist: GpuJson, version: u64, limits: GenericGpuLimit) -> Self {
        let clock_limits = persist.clock_limits.clone().map(|lim| MinMax {
            min: lim.min,
            max: lim.max,
        });
        Self {
            persist,
            version,
            limits,
            clock_limits,
        }
    }

    fn from_limits(limits: GenericGpuLimit) -> Self {
        Self {
            persist: GpuJson {
                fast_ppt: None,
                slow_ppt: None,
                tdp: None,
                tdp_boost: None,
                clock_limits: None,
                memory_clock: None,
                root: None,
            },
            version: 0,
            limits,
            clock_limits: None,
        }
    }
}

impl Into<GpuJson> for Gpu {
    #[inline]
    fn into(self) -> GpuJson {
        self.persist
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Gpu::on_set(self)");
        Ok(())
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Gpu::on_resume(self)");
        Ok(())
    }
}

impl crate::settings::OnPowerEvent for Gpu {}

impl crate::settings::OnLoad for Gpu {
    fn on_load(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Gpu::on_load(self)");
        Ok(())
    }
}

impl crate::settings::OnUnload for Gpu {
    fn on_unload(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Gpu::on_unload(self)");
        Ok(())
    }
}

impl TGpu for Gpu {
    fn limits(&self) -> crate::api::GpuLimits {
        log::debug!("dev_mode_Gpu::limits(self) -> {{...}}");
        let ppt_divisor = self.limits.ppt_divisor.unwrap_or(1_000_000);
        let tdp_divisor = self.limits.tdp_divisor.unwrap_or(1_000_000);
        let limit_struct = crate::api::GpuLimits {
            fast_ppt_limits: self.limits.fast_ppt.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(11_000_000) / ppt_divisor,
                max: lim.max.unwrap_or(42_000_000) / ppt_divisor,
            }),
            fast_ppt_default: self.limits.fast_ppt_default.or_else(|| self.limits.fast_ppt.and_then(|x| x.max)).unwrap_or(2_000_000) / ppt_divisor,
            slow_ppt_limits: self.limits.slow_ppt.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(7_000_000) / ppt_divisor,
                max: lim.max.unwrap_or(69_000_000) / ppt_divisor,
            }),
            slow_ppt_default: self.limits.slow_ppt_default.or_else(|| self.limits.slow_ppt.and_then(|x| x.max)).unwrap_or(3_000_000) / ppt_divisor,
            ppt_step: self.limits.ppt_step.unwrap_or(1),
            tdp_limits: self.limits.tdp.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(11_000_000) / tdp_divisor,
                max: lim.max.unwrap_or(69_000_000) / tdp_divisor,
            }),
            tdp_boost_limits: self.limits.tdp_boost.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(7_000_000) / tdp_divisor,
                max: lim.max.unwrap_or(69_000_000) / tdp_divisor,
            }),
            tdp_step: self.limits.tdp_step.unwrap_or(1),
            clock_min_limits: self.limits.clock_min.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(1100),
                max: lim.max.unwrap_or(6900),
            }),
            clock_max_limits: self.limits.clock_max.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(1100),
                max: lim.max.unwrap_or(4200),
            }),
            clock_step: self.limits.clock_step.unwrap_or(100),
            memory_control: self.limits.memory_clock.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(100),
                max: lim.max.unwrap_or(1100),
            }),
            memory_step: self.limits.memory_clock_step.unwrap_or(400),
        };
        log::debug!("dev_mode_Gpu::limits(self) -> {}", serde_json::to_string_pretty(&limit_struct).unwrap());
        limit_struct
    }

    fn json(&self) -> crate::persist::GpuJson {
        log::debug!("dev_mode_Gpu::json(self) -> {{...}}");
        self.clone().into()
    }

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>) {
        log::debug!(
            "dev_mode_Gpu::ppt(self, fast: {:?}, slow: {:?})",
            fast,
            slow
        );
        self.persist.fast_ppt = fast;
        self.persist.slow_ppt = slow;
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        log::debug!(
            "dev_mode_Gpu::get_ppt(self) -> (fast: {:?}, slow: {:?})",
            self.persist.fast_ppt,
            self.persist.slow_ppt
        );
        (self.persist.fast_ppt, self.persist.slow_ppt)
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        log::debug!("dev_mode_Gpu::clock_limits(self, {:?})", limits);
        self.clock_limits = limits;
        self.persist.clock_limits =
            self.clock_limits
                .clone()
                .map(|lim| crate::persist::MinMaxJson {
                    max: lim.max,
                    min: lim.min,
                });
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        log::debug!(
            "dev_mode_Gpu::get_clock_limits(self) -> {:?}",
            self.clock_limits.as_ref()
        );
        self.clock_limits.as_ref()
    }

    fn memory_clock(&mut self, speed: Option<u64>) {
        log::debug!("dev_mode_Gpu::memory_clock(self, {:?})", speed);
        self.persist.memory_clock = speed;
    }

    fn get_memory_clock(&self) -> Option<u64> {
        log::debug!(
            "dev_mode_Gpu::memory_clock(self) -> {:?}",
            self.persist.memory_clock
        );
        self.persist.memory_clock
    }

    fn provider(&self) -> crate::persist::DriverJson {
        log::debug!("dev_mode_Gpu::provider(self) -> DevMode");
        crate::persist::DriverJson::DevMode
    }
}
