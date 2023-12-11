use std::convert::Into;

use limits_core::json_v2::GenericGpuLimit;
use sysfuss::{BasicEntityPath, SysEntity};

use crate::api::RangeLimit;
use crate::persist::GpuJson;
use crate::settings::{TGpu, ProviderBuilder};
use crate::settings::{min_max_from_json, MinMax};
use crate::settings::{OnResume, OnSet, SettingError};

#[derive(Debug, Clone)]
pub struct Gpu {
    pub slow_memory: bool,
    pub fast_ppt: Option<u64>,
    pub slow_ppt: Option<u64>,
    pub clock_limits: Option<MinMax<u64>>,
    pub limits: GenericGpuLimit,
    sysfs: BasicEntityPath,
}

impl Gpu {
    /*#[inline]
    pub fn from_json(_other: GpuJson, _version: u64) -> Self {
        Self {
            slow_memory: false,
        }
    }*/

    /*pub fn system_default() -> Self {
        Self {
            slow_memory: false,
        }
    }*/

    fn find_card_sysfs(root: Option<impl AsRef<std::path::Path>>) -> BasicEntityPath {
        let root = crate::settings::util::root_or_default_sysfs(root);
        match root.class("drm", crate::settings::util::always_satisfied) {
            Ok(mut iter) => {
                iter.next()
                    .unwrap_or_else(|| {
                        log::error!("Failed to find generic gpu drm in sysfs (no results), using naive fallback");
                        BasicEntityPath::new(root.as_ref().join("sys/class/drm/card0"))
                    })
            },
            Err(e) => {
                log::error!("Failed to find generic gpu drm in sysfs ({}), using naive fallback", e);
                BasicEntityPath::new(root.as_ref().join("sys/class/drm/card0"))
            }
        }
    }
}

impl ProviderBuilder<GpuJson, GenericGpuLimit> for Gpu {
    fn from_json_and_limits(persistent: GpuJson, version: u64, limits: GenericGpuLimit) -> Self {
        let clock_lims = if limits.clock_min.is_some() && limits.clock_max.is_some() {
            persistent.clock_limits.map(|x| min_max_from_json(x, version))
        } else {
            None
        };
        Self {
            slow_memory: false,
            fast_ppt: if limits.fast_ppt.is_some() {
                persistent.fast_ppt
            } else {
                None
            },
            slow_ppt: if limits.slow_ppt.is_some() {
                persistent.slow_ppt
            } else {
                None
            },
            clock_limits: clock_lims,
            limits,
            sysfs: Self::find_card_sysfs(persistent.root)
        }
    }

    fn from_limits(limits: GenericGpuLimit) -> Self {
        Self {
            slow_memory: false,
            fast_ppt: None,
            slow_ppt: None,
            clock_limits: None,
            limits,
            sysfs: Self::find_card_sysfs(None::<&'static str>),
        }
    }
}

impl Into<GpuJson> for Gpu {
    #[inline]
    fn into(self) -> GpuJson {
        GpuJson {
            fast_ppt: self.fast_ppt,
            slow_ppt: self.slow_ppt,
            clock_limits: self.clock_limits.map(|x| x.into()),
            slow_memory: false,
            root: self.sysfs.root().and_then(|p| p.as_ref().to_str().map(|s| s.to_owned()))
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
            fast_ppt_limits: self
                .limits
                .fast_ppt
                .clone()
                .map(|x| RangeLimit::new(x.min.unwrap_or(0), x.max.unwrap_or(15)))
                .map(|mut x| if let Some(ppt_divisor) = self.limits.ppt_divisor {
                    x.min /= ppt_divisor;
                    x.max /= ppt_divisor;
                    x
                } else {x}),
            slow_ppt_limits: self
                .limits
                .slow_ppt
                .clone()
                .map(|x| RangeLimit::new(x.min.unwrap_or(0), x.max.unwrap_or(15)))
                .map(|mut x| if let Some(ppt_divisor) = self.limits.ppt_divisor {
                    x.min /= ppt_divisor;
                    x.max /= ppt_divisor;
                    x
                } else {x}),
            ppt_step: self.limits.ppt_step.unwrap_or(1),
            tdp_limits: self
                .limits
                .tdp
                .clone()
                .map(|x| RangeLimit::new(x.min.unwrap_or(0), x.max.unwrap_or(15_000_000))),
            tdp_boost_limits: self
                .limits
                .tdp_boost
                .clone()
                .map(|x| RangeLimit::new(x.min.unwrap_or(0), x.max.unwrap_or(15_000_000))),
            tdp_step: self.limits.tdp_step.unwrap_or(42),
            clock_min_limits: self
                .limits
                .clock_min
                .clone()
                .map(|x| RangeLimit::new(x.min.unwrap_or(0), x.max.unwrap_or(3_000))),
            clock_max_limits: self
                .limits
                .clock_max
                .clone()
                .map(|x| RangeLimit::new(x.min.unwrap_or(0), x.max.unwrap_or(3_000))),
            clock_step: self.limits.clock_step.unwrap_or(100),
            memory_control_capable: false,
        }
    }

    fn json(&self) -> crate::persist::GpuJson {
        self.clone().into()
    }

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>) {
        if let Some(fast_lims) = &self.limits.fast_ppt {
            self.fast_ppt = fast.map(|x| if let Some(ppt_divisor) = self.limits.ppt_divisor { x * ppt_divisor } else { x })
            .map(|x| {
                x.clamp(
                    fast_lims.min.unwrap_or(0),
                    fast_lims.max.unwrap_or(u64::MAX),
                )
            });
        }
        if let Some(slow_lims) = &self.limits.slow_ppt {
            self.slow_ppt = slow.map(|x| if let Some(ppt_divisor) = self.limits.ppt_divisor { x * ppt_divisor } else { x })
            .map(|x| {
                x.clamp(
                    slow_lims.min.unwrap_or(0),
                    slow_lims.max.unwrap_or(u64::MAX),
                )
            });
        }
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        (self.fast_ppt, self.slow_ppt)
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        if let Some(clock_min) = &self.limits.clock_min {
            if let Some(clock_max) = &self.limits.clock_max {
                self.clock_limits = limits.map(|mut x| {
                    x.min = x.min.clamp(clock_min.min, clock_min.max);
                    x.max = x.max.clamp(clock_max.max, clock_max.max);
                    x
                });
            }
        }
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.clock_limits.as_ref()
    }

    fn slow_memory(&mut self) -> &mut bool {
        &mut self.slow_memory
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Generic
    }
}
