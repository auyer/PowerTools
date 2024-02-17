use libryzenadj::RyzenAdj;
use std::sync::Mutex;

use crate::persist::GpuJson;
use crate::settings::generic::Gpu as GenericGpu;
use crate::settings::MinMax;
use crate::settings::{OnResume, OnSet, SettingError, SettingVariant};
use crate::settings::{ProviderBuilder, TGpu};

fn msg_or_err<D: std::fmt::Display, E: std::fmt::Display>(
    output: &mut String,
    msg: &str,
    result: Result<D, E>,
) {
    use std::fmt::Write;
    match result {
        Ok(val) => writeln!(output, "{}: {}", msg, val).unwrap(),
        Err(e) => writeln!(output, "{} failed: {}", msg, e).unwrap(),
    }
}

fn log_capabilities(ryzenadj: &RyzenAdj) {
    log::info!(
        "RyzenAdj v{}.{}.{}",
        libryzenadj::libryzenadj_sys::RYZENADJ_REVISION_VER,
        libryzenadj::libryzenadj_sys::RYZENADJ_MAJOR_VER,
        libryzenadj::libryzenadj_sys::RYZENADJ_MINIOR_VER
    );
    #[cfg(feature = "experimental")]
    if let Some(x) = ryzenadj.get_init_table_err() {
        log::warn!("RyzenAdj table init error: {}", x);
    }
    let mut log_msg = String::new();
    msg_or_err(&mut log_msg, "bios version", ryzenadj.get_bios_if_ver());
    msg_or_err(
        &mut log_msg,
        "refresh",
        ryzenadj.refresh().map(|_| "success"),
    );
    msg_or_err(
        &mut log_msg,
        "CPU family",
        ryzenadj.get_cpu_family().map(|fam| {
            let fam_dbg = format!("{:?}", fam);
            format!("{} (#{})", fam_dbg, fam as i32)
        }),
    );
    msg_or_err(
        &mut log_msg,
        "get_fast_value (PPT)",
        ryzenadj.get_fast_value(),
    );
    msg_or_err(
        &mut log_msg,
        "get_slow_value (PPT)",
        ryzenadj.get_slow_value(),
    );
    msg_or_err(&mut log_msg, "get_gfx_clk", ryzenadj.get_gfx_clk());
    msg_or_err(&mut log_msg, "get_gfx_volt", ryzenadj.get_gfx_volt());

    log::info!("RyzenAdj GPU info:\n{}", log_msg);
}

fn ryzen_adj_or_log() -> Option<Mutex<RyzenAdj>> {
    match RyzenAdj::new() {
        Ok(x) => {
            log_capabilities(&x);
            Some(Mutex::new(x))
        }
        Err(e) => {
            log::error!("RyzenAdj init error: {}", e);
            None
        }
    }
}

unsafe impl Send for Gpu {} // implementor (RyzenAdj) may be unsafe

//#[derive(Debug)]
pub struct Gpu {
    generic: GenericGpu,
    implementor: Option<Mutex<RyzenAdj>>,
    state: crate::state::generic::Gpu, // NOTE this is re-used for simplicity
}

impl std::fmt::Debug for Gpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gpu")
            .field("generic", &self.generic)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl ProviderBuilder<GpuJson, limits_core::json_v2::GenericGpuLimit> for Gpu {
    fn from_limits(limits: limits_core::json_v2::GenericGpuLimit) -> Self {
        Self {
            generic: GenericGpu::from_limits(limits),
            implementor: ryzen_adj_or_log(),
            state: Default::default(),
        }
    }

    fn from_json_and_limits(
        other: GpuJson,
        version: u64,
        limits: limits_core::json_v2::GenericGpuLimit,
    ) -> Self {
        Self {
            generic: GenericGpu::from_json_and_limits(other, version, limits),
            implementor: ryzen_adj_or_log(),
            state: Default::default(),
        }
    }
}

impl Gpu {
    fn set_all(&mut self) -> Result<(), Vec<SettingError>> {
        let mutex = match &self.implementor {
            Some(x) => x,
            None => {
                return Err(vec![SettingError {
                    msg: "RyzenAdj unavailable".to_owned(),
                    setting: SettingVariant::Gpu,
                }]);
            }
        };
        let lock = match mutex.lock() {
            Ok(x) => x,
            Err(e) => {
                return Err(vec![SettingError {
                    msg: format!("RyzenAdj lock acquire failed: {}", e),
                    setting: SettingVariant::Gpu,
                }]);
            }
        };
        let mut errors = Vec::new();
        if let Some(fast_ppt) = &self.generic.fast_ppt {
            if self.state.old_fast_ppt.is_none() {
                match lock.get_fast_value() {
                    Ok(val) => self.state.old_fast_ppt = Some(val as _),
                    Err(e) => errors.push(SettingError {
                        msg: format!("RyzenAdj get_fast_value() err: {}", e),
                        setting: SettingVariant::Gpu,
                    }),
                }
            }
            lock.set_fast_limit(*fast_ppt as _)
                .map_err(|e| SettingError {
                    msg: format!("RyzenAdj set_fast_limit({}) err: {}", *fast_ppt, e),
                    setting: SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| errors.push(e));
        } else if let Some(fast_ppt) = &self.state.old_fast_ppt {
            lock.set_fast_limit(*fast_ppt as _)
                .map_err(|e| SettingError {
                    msg: format!("RyzenAdj set_fast_limit({}) err: {}", *fast_ppt, e),
                    setting: SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| errors.push(e));
            self.state.old_fast_ppt = None;
        }
        if let Some(slow_ppt) = &self.generic.slow_ppt {
            if self.state.old_slow_ppt.is_none() {
                match lock.get_slow_value() {
                    Ok(val) => self.state.old_fast_ppt = Some(val as _),
                    Err(e) => errors.push(SettingError {
                        msg: format!("RyzenAdj get_slow_value() err: {}", e),
                        setting: SettingVariant::Gpu,
                    }),
                }
            }
            lock.set_slow_limit(*slow_ppt as _)
                .map_err(|e| SettingError {
                    msg: format!("RyzenAdj set_slow_limit({}) err: {}", *slow_ppt, e),
                    setting: SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| errors.push(e));
        } else if let Some(slow_ppt) = &self.state.old_slow_ppt {
            lock.set_slow_limit(*slow_ppt as _)
                .map_err(|e| SettingError {
                    msg: format!("RyzenAdj set_slow_limit({}) err: {}", *slow_ppt, e),
                    setting: SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| errors.push(e));
            self.state.old_slow_ppt = None;
        }
        if let Some(clock_limits) = &self.generic.clock_limits {
            self.state.clock_limits_set = true;
            if let Some(max) = clock_limits.max {
                lock.set_max_gfxclk_freq(max as _)
                    .map_err(|e| SettingError {
                        msg: format!("RyzenAdj set_max_gfxclk_freq({}) err: {}", max, e),
                        setting: SettingVariant::Gpu,
                    })
                    .unwrap_or_else(|e| errors.push(e));
            }
            if let Some(min) = clock_limits.min {
                lock.set_min_gfxclk_freq(min as _)
                    .map_err(|e| SettingError {
                        msg: format!("RyzenAdj set_min_gfxclk_freq({}) err: {}", min, e),
                        setting: SettingVariant::Gpu,
                    })
                    .unwrap_or_else(|e| errors.push(e));
            }
        } else if self.state.clock_limits_set {
            self.state.clock_limits_set = false;
            let limits = self.generic.limits();
            if let Some(min_limits) = limits.clock_min_limits {
                if let Some(max_limits) = limits.clock_max_limits {
                    lock.set_max_gfxclk_freq(max_limits.max as _)
                        .map_err(|e| SettingError {
                            msg: format!(
                                "RyzenAdj set_max_gfxclk_freq({}) err: {}",
                                max_limits.max, e
                            ),
                            setting: SettingVariant::Gpu,
                        })
                        .unwrap_or_else(|e| errors.push(e));
                    lock.set_min_gfxclk_freq(min_limits.min as _)
                        .map_err(|e| SettingError {
                            msg: format!(
                                "RyzenAdj set_min_gfxclk_freq({}) err: {}",
                                min_limits.min, e
                            ),
                            setting: SettingVariant::Gpu,
                        })
                        .unwrap_or_else(|e| errors.push(e));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn resume_all(&self) -> Result<(), Vec<SettingError>> {
        // like set_all() but without updating state
        // -- assumption: state is already up to date
        let mutex = match &self.implementor {
            Some(x) => x,
            None => {
                return Err(vec![SettingError {
                    msg: "RyzenAdj unavailable".to_owned(),
                    setting: SettingVariant::Gpu,
                }]);
            }
        };
        let lock = match mutex.lock() {
            Ok(x) => x,
            Err(e) => {
                return Err(vec![SettingError {
                    msg: format!("RyzenAdj lock acquire failed: {}", e),
                    setting: SettingVariant::Gpu,
                }]);
            }
        };
        let mut errors = Vec::new();
        if let Some(fast_ppt) = &self.generic.fast_ppt {
            lock.set_fast_limit(*fast_ppt as _)
                .map_err(|e| SettingError {
                    msg: format!("RyzenAdj set_fast_limit({}) err: {}", *fast_ppt, e),
                    setting: SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| errors.push(e));
        }
        if let Some(slow_ppt) = &self.generic.slow_ppt {
            lock.set_slow_limit(*slow_ppt as _)
                .map_err(|e| SettingError {
                    msg: format!("RyzenAdj set_slow_limit({}) err: {}", *slow_ppt, e),
                    setting: SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| errors.push(e));
        }
        if let Some(clock_limits) = &self.generic.clock_limits {
            if let Some(max) = clock_limits.max {
                lock.set_max_gfxclk_freq(max as _)
                    .map_err(|e| SettingError {
                        msg: format!("RyzenAdj set_max_gfxclk_freq({}) err: {}", max, e),
                        setting: SettingVariant::Gpu,
                    })
                    .unwrap_or_else(|e| errors.push(e));
            }
            if let Some(min) = clock_limits.min {
                lock.set_min_gfxclk_freq(min as _)
                    .map_err(|e| SettingError {
                        msg: format!("RyzenAdj set_min_gfxclk_freq({}) err: {}", min, e),
                        setting: SettingVariant::Gpu,
                    })
                    .unwrap_or_else(|e| errors.push(e));
            }
        }
        Ok(())
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        self.generic.on_resume()?;
        self.resume_all()
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.generic.on_set()?;
        self.set_all()
    }
}

impl crate::settings::OnPowerEvent for Gpu {}

impl crate::settings::OnLoad for Gpu {
    fn on_load(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl crate::settings::OnUnload for Gpu {
    fn on_unload(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

fn bad_gpu_limits() -> crate::api::GpuLimits {
    crate::api::GpuLimits {
        fast_ppt_limits: None,
        fast_ppt_default: 1,
        slow_ppt_limits: None,
        slow_ppt_default: 1,
        ppt_step: 1,
        tdp_limits: None,
        tdp_boost_limits: None,
        tdp_step: 1,
        clock_min_limits: None,
        clock_max_limits: None,
        clock_step: 100,
        memory_control: None,
        memory_step: 400,
    }
}

impl TGpu for Gpu {
    fn limits(&self) -> crate::api::GpuLimits {
        if self.implementor.is_some() {
            // NOTE: since set functions may succeed when gets do not, there is no good way to (automatically) check whether things are working
            self.generic.limits()
        } else {
            bad_gpu_limits()
        }
    }

    fn json(&self) -> crate::persist::GpuJson {
        self.generic.json()
    }

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>) {
        self.generic.ppt(fast, slow)
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        self.generic.get_ppt()
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        self.generic.clock_limits(limits)
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.generic.get_clock_limits()
    }

    fn memory_clock(&mut self, speed: Option<u64>) {
        self.generic.memory_clock(speed)
    }

    fn get_memory_clock(&self) -> Option<u64> {
        self.generic.get_memory_clock()
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::GenericAMD
    }
}
