use std::convert::Into;

use sysfuss::{BasicEntityPath, HwMonPath, SysEntity, capability::attributes, SysEntityAttributes, SysEntityAttributesExt, SysAttribute};

use limits_core::json_v2::GenericGpuLimit;

use super::POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT;
use crate::api::RangeLimit;
use crate::persist::GpuJson;
use crate::settings::{TGpu, ProviderBuilder};
use crate::settings::{min_max_from_json, MinMax};
use crate::settings::{OnResume, OnSet, SettingError};

// usually in /sys/class/hwmon/hwmon4/<attribute>
const SLOW_PPT_ATTRIBUTE: sysfuss::HwMonAttribute = sysfuss::HwMonAttribute::custom("power1_cap");
const FAST_PPT_ATTRIBUTE: sysfuss::HwMonAttribute = sysfuss::HwMonAttribute::custom("power2_cap");

#[derive(Debug, Clone)]
pub struct Gpu {
    pub fast_ppt: Option<u64>,
    pub slow_ppt: Option<u64>,
    pub clock_limits: Option<MinMax<u64>>,
    pub memory_clock: Option<u64>,
    limits: GenericGpuLimit,
    state: crate::state::steam_deck::Gpu,
    sysfs_card: BasicEntityPath,
    sysfs_hwmon: HwMonPath
}

// same as CPU
//const GPU_CLOCK_LIMITS_PATH: &str = "/sys/class/drm/card0/device/pp_od_clk_voltage";
//const GPU_MEMORY_DOWNCLOCK_PATH: &str = "/sys/class/drm/card0/device/pp_dpm_fclk";

const GPU_CLOCK_LIMITS_ATTRIBUTE: &str = "device/pp_od_clk_voltage";
const GPU_MEMORY_DOWNCLOCK_ATTRIBUTE: &str = "device/pp_dpm_fclk";

const CARD_EXTENSIONS: &[&'static str] = &[
    GPU_CLOCK_LIMITS_ATTRIBUTE,
    GPU_MEMORY_DOWNCLOCK_ATTRIBUTE,
    super::DPM_FORCE_LIMITS_ATTRIBUTE,
];

enum ClockType {
    Min = 0,
    Max = 1,
}

const MAX_CLOCK: u64 = 1600;
const MIN_CLOCK: u64 = 200;
const MAX_MEMORY_CLOCK: u64 = 800;
const MIN_MEMORY_CLOCK: u64 = 400;
const MAX_FAST_PPT: u64 = 30_000_000;
const MIN_FAST_PPT: u64 = 1_000_000;
const MAX_SLOW_PPT: u64 = 29_000_000;
const MIN_SLOW_PPT: u64 = 1_000_000;
const MIDDLE_PPT: u64 = 15_000_000;
const PPT_DIVISOR: u64 = 1_000;

impl Gpu {
    fn find_card_sysfs(root: Option<impl AsRef<std::path::Path>>) -> BasicEntityPath {
        let root = crate::settings::util::root_or_default_sysfs(root);
        match root.class("drm", attributes(crate::settings::util::CARD_NEEDS.into_iter().map(|s| s.to_string()))) {
            Ok(iter) => {
                let card = iter
                    .filter(|ent| if let Ok(name) = ent.name() { name.starts_with("card")} else { false })
                    .filter(|ent| super::util::card_also_has(ent, CARD_EXTENSIONS))
                    .next()
                    .unwrap_or_else(|| {
                        log::error!("Failed to find SteamDeck gpu drm in sysfs (no results), using naive fallback");
                        BasicEntityPath::new(root.as_ref().join("sys/class/drm/card0"))
                    });
                log::info!("Found SteamDeck gpu drm in sysfs: {}", card.as_ref().display());
                card
            },
            Err(e) => {
                log::error!("Failed to find SteamDeck gpu drm in sysfs ({}), using naive fallback", e);
                BasicEntityPath::new(root.as_ref().join("sys/class/drm/card0"))
            }
        }
    }

    fn find_hwmon_sysfs(root: Option<impl AsRef<std::path::Path>>) -> HwMonPath {
        let root = crate::settings::util::root_or_default_sysfs(root);
        let hwmon = root.hwmon_by_name(super::util::GPU_HWMON_NAME).unwrap_or_else(|e| {
            log::error!("Failed to find SteamDeck gpu hwmon in sysfs ({}), using naive fallback", e);
            root.hwmon_by_index(4)
        });
        log::info!("Found SteamDeck gpu hwmon {} in sysfs: {}", super::util::GPU_HWMON_NAME, hwmon.as_ref().display());
        hwmon
    }

    fn set_clock_limit(&self, speed: u64, mode: ClockType) -> Result<(), SettingError> {
        let payload = format!("s {} {}\n", mode as u8, speed);
        let path = GPU_CLOCK_LIMITS_ATTRIBUTE.path(&self.sysfs_card);
        self.sysfs_card.set(GPU_CLOCK_LIMITS_ATTRIBUTE.to_owned(), &payload).map_err(|e| {
            SettingError {
                msg: format!("Failed to write `{}` to `{}`: {}", &payload, path.display(), e),
                setting: crate::settings::SettingVariant::Gpu,
            }
        })
    }

    fn set_confirm(&self) -> Result<(), SettingError> {
        let path = GPU_CLOCK_LIMITS_ATTRIBUTE.path(&self.sysfs_card);
        self.sysfs_card.set(GPU_CLOCK_LIMITS_ATTRIBUTE.to_owned(), "c\n").map_err(|e| {
            SettingError {
                msg: format!("Failed to write `c` to `{}`: {}", path.display(), e),
                setting: crate::settings::SettingVariant::Gpu,
            }
        })
    }

    fn is_memory_clock_maxed(&self) -> bool {
        if let Some(clock) = &self.memory_clock {
            if let Some(limit) = &self.limits.memory_clock {
                if let Some(limit) = &limit.max {
                    if let Some(step) = &self.limits.memory_clock_step {
                        log::debug!("chosen_clock: {}, limit_clock: {}, step: {}", clock, limit, step);
                        return clock > &(limit - step);
                    } else {
                        log::debug!("chosen_clock: {}, limit_clock: {}", clock, limit);
                        return clock == limit;
                    }
                }
            }
        }
        true
    }

    fn quantize_memory_clock(&self, clock: u64) -> u64 {
        if let Ok(f) = self.sysfs_card.read_value(GPU_MEMORY_DOWNCLOCK_ATTRIBUTE.to_owned()) {
            let options = parse_pp_dpm_fclk(&String::from_utf8_lossy(&f));
            // round (and find) nearest valid clock step
            // roughly price is right strategy (clock step will always be lower or equal to chosen)
            for i in 0..options.len() {
                let (current_val_opt, current_speed_opt) = &options[i];
                let current_speed_opt = *current_speed_opt as u64;
                if clock == current_speed_opt {
                    return *current_val_opt as _;
                } else if current_speed_opt > clock {
                    if i == 0 {
                        return *current_val_opt as _;
                    } else {
                        return options[i-1].0 as _;
                    }
                }
            }
            options[options.len() - 1].0 as _
        } else {
            self.is_memory_clock_maxed() as u64
        }
    }

    fn build_memory_clock_payload(&self, clock: u64) -> String {
        let max_val = self.quantize_memory_clock(clock);
        match max_val {
            0 => "0\n".to_owned(),
            max_val => {
                use std::fmt::Write;
                let mut payload = String::from("0");
                for i in 1..max_val {
                    write!(payload, " {}", i).expect("Failed to write to memory payload (should be infallible!?)");
                }
                write!(payload, " {}\n", max_val).expect("Failed to write to memory payload (should be infallible!?)");
                payload
            }
        }
    }

    fn set_clocks(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        if let Some(clock_limits) = &self.clock_limits {
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_gpu(true);
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.enforce_level(&self.sysfs_card)?;
            // set clock limits
            self.state.clock_limits_set = true;
            // max clock
            if let Some(max) = clock_limits.max {
                self.set_clock_limit(max, ClockType::Max).unwrap_or_else(|e| errors.push(e));
            }
            // min clock
            if let Some(min) = clock_limits.min {
                self.set_clock_limit(min, ClockType::Min).unwrap_or_else(|e| errors.push(e));
            }

            self.set_confirm().unwrap_or_else(|e| errors.push(e));
        } else if self.state.clock_limits_set
            || (self.state.is_resuming && !self.limits.skip_resume_reclock)
            || POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.needs_manual()
        {
            self.state.clock_limits_set = false;
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_gpu(!self.is_memory_clock_maxed());
            if POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.needs_manual() {
                POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.enforce_level(&self.sysfs_card)?;
                // disable manual clock limits
                // max clock
                self.set_clock_limit(self.limits.clock_max.and_then(|lim| lim.max).unwrap_or(MAX_CLOCK), ClockType::Max)
                    .unwrap_or_else(|e| errors.push(e));
                // min clock
                self.set_clock_limit(self.limits.clock_min.and_then(|lim| lim.min).unwrap_or(MIN_CLOCK), ClockType::Min)
                    .unwrap_or_else(|e| errors.push(e));

                self.set_confirm().unwrap_or_else(|e| errors.push(e));
            } else {
                POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT
                    .enforce_level(&self.sysfs_card)
                    .unwrap_or_else(|mut e| errors.append(&mut e));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn set_memory_speed(&self, clock: u64) -> Result<(), SettingError> {
        let path = GPU_MEMORY_DOWNCLOCK_ATTRIBUTE.path(&self.sysfs_card);
        let payload = self.build_memory_clock_payload(clock);
        log::debug!("Generated payload for gpu fclk (memory): `{}` (is maxed? {})", payload, self.is_memory_clock_maxed());
        self.sysfs_card.set(GPU_MEMORY_DOWNCLOCK_ATTRIBUTE.to_owned(), payload).map_err(|e| {
            SettingError {
                msg: format!("Failed to write to `{}`: {}", path.display(), e),
                setting: crate::settings::SettingVariant::Gpu,
            }
        })
    }

    fn set_force_performance_related(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_gpu(!self.is_memory_clock_maxed() || self.clock_limits.is_some());
        POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT
                .enforce_level(&self.sysfs_card)
                .unwrap_or_else(|mut e| errors.append(&mut e));
        // enable/disable downclock of GPU memory (to 400Mhz?)
        self.set_memory_speed(
            self.memory_clock
                .or_else(|| self.limits.memory_clock
                    .map(|lim| lim.max.unwrap_or(MAX_MEMORY_CLOCK))
            ).unwrap_or(MAX_MEMORY_CLOCK)
        ).unwrap_or_else(|e| errors.push(e));
        self.set_clocks()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        // commit changes (if no errors have already occured)
        if errors.is_empty() {
            if !self.is_memory_clock_maxed() || self.clock_limits.is_some() {
                self.set_confirm().map_err(|e| {
                    errors.push(e);
                    errors
                })
            } else {
                Ok(())
            }
        } else {
            Err(errors)
        }
    }

    fn set_all(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        // set fast PPT
        if let Some(fast_ppt) = &self.fast_ppt {
            self.state.fast_ppt_set = true;
            self.sysfs_hwmon.set(FAST_PPT_ATTRIBUTE, fast_ppt)
                .map_err(|e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{:?}`: {}",
                        fast_ppt, FAST_PPT_ATTRIBUTE, e
                    ),
                    setting: crate::settings::SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| {
                    errors.push(e);
                });
        } else if self.state.fast_ppt_set {
            self.state.fast_ppt_set = false;
            let fast_ppt = self.limits.fast_ppt_default.unwrap_or(MIDDLE_PPT);
            self.sysfs_hwmon.set(FAST_PPT_ATTRIBUTE, fast_ppt)
                .map_err(|e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{:?}`: {}",
                        fast_ppt, FAST_PPT_ATTRIBUTE, e
                    ),
                    setting: crate::settings::SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| {
                    errors.push(e);
                });
        }
        // set slow PPT
        if let Some(slow_ppt) = &self.slow_ppt {
            self.state.slow_ppt_set = true;
            self.sysfs_hwmon.set(SLOW_PPT_ATTRIBUTE, slow_ppt)
                .map_err(|e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{:?}`: {}",
                        slow_ppt, SLOW_PPT_ATTRIBUTE, e
                    ),
                    setting: crate::settings::SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| {
                    errors.push(e);
                });
        } else if self.state.slow_ppt_set {
            self.state.slow_ppt_set = false;
            let slow_ppt = self.limits.slow_ppt_default.unwrap_or(MIDDLE_PPT);
            self.sysfs_hwmon.set(SLOW_PPT_ATTRIBUTE, slow_ppt)
                .map_err(|e| SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{:?}`: {}",
                        slow_ppt, SLOW_PPT_ATTRIBUTE, e
                    ),
                    setting: crate::settings::SettingVariant::Gpu,
                })
                .unwrap_or_else(|e| {
                    errors.push(e);
                });
        }
        self.set_force_performance_related()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn clamp_all(&mut self) {
        if let Some(fast_ppt) = &mut self.fast_ppt {
            *fast_ppt = (*fast_ppt).clamp(self.limits.fast_ppt.and_then(|lim| lim.min).unwrap_or(MIN_FAST_PPT), self.limits.fast_ppt.and_then(|lim| lim.max).unwrap_or(MAX_FAST_PPT));
        }
        if let Some(slow_ppt) = &mut self.slow_ppt {
            *slow_ppt = (*slow_ppt).clamp(self.limits.slow_ppt.and_then(|lim| lim.min).unwrap_or(MIN_SLOW_PPT), self.limits.slow_ppt.and_then(|lim| lim.max).unwrap_or(MAX_SLOW_PPT));
        }
        if let Some(clock_limits) = &mut self.clock_limits {
            if let Some(min) = clock_limits.min {
                clock_limits.min =
                    Some(min.clamp(self.limits.clock_min.and_then(|lim| lim.min).unwrap_or(MIN_CLOCK), self.limits.clock_min.and_then(|lim| lim.max).unwrap_or(MAX_CLOCK)));
            }
            if let Some(max) = clock_limits.max {
                clock_limits.max =
                    Some(max.clamp(self.limits.clock_max.and_then(|lim| lim.min).unwrap_or(MIN_CLOCK), self.limits.clock_max.and_then(|lim| lim.max).unwrap_or(MAX_CLOCK)));
            }
        }
        if let Some(mem_clock) = self.memory_clock {
            self.memory_clock = Some(mem_clock.clamp(self.limits.memory_clock.and_then(|lim| lim.min).unwrap_or(MIN_MEMORY_CLOCK), self.limits.memory_clock.and_then(|lim| lim.max).unwrap_or(MAX_MEMORY_CLOCK)));
        }
    }
}

impl Into<GpuJson> for Gpu {
    #[inline]
    fn into(self) -> GpuJson {
        GpuJson {
            fast_ppt: self.fast_ppt,
            slow_ppt: self.slow_ppt,
            tdp: None,
            tdp_boost: None,
            clock_limits: self.clock_limits.map(|x| x.into()),
            memory_clock: self.memory_clock,
            root: self.sysfs_card.root().or(self.sysfs_hwmon.root()).and_then(|p| p.as_ref().to_str().map(|r| r.to_owned()))
        }
    }
}

impl ProviderBuilder<GpuJson, GenericGpuLimit> for Gpu {
    fn from_json_and_limits(persistent: GpuJson, version: u64, limits: GenericGpuLimit) -> Self {
        match version {
            0 => Self {
                fast_ppt: persistent.fast_ppt,
                slow_ppt: persistent.slow_ppt,
                clock_limits: persistent.clock_limits.map(|x| min_max_from_json(x, version)),
                memory_clock: persistent.memory_clock,
                limits: limits,
                state: crate::state::steam_deck::Gpu::default(),
                sysfs_card: Self::find_card_sysfs(persistent.root.clone()),
                sysfs_hwmon: Self::find_hwmon_sysfs(persistent.root),
            },
            _ => Self {
                fast_ppt: persistent.fast_ppt,
                slow_ppt: persistent.slow_ppt,
                clock_limits: persistent.clock_limits.map(|x| min_max_from_json(x, version)),
                memory_clock: persistent.memory_clock,
                limits: limits,
                state: crate::state::steam_deck::Gpu::default(),
                sysfs_card: Self::find_card_sysfs(persistent.root.clone()),
                sysfs_hwmon: Self::find_hwmon_sysfs(persistent.root),
            },
        }
    }

    fn from_limits(limits: GenericGpuLimit) -> Self {
        Self {
            fast_ppt: None,
            slow_ppt: None,
            clock_limits: None,
            memory_clock: None,
            limits: limits,
            state: crate::state::steam_deck::Gpu::default(),
            sysfs_card: Self::find_card_sysfs(None::<&'static str>),
            sysfs_hwmon: Self::find_hwmon_sysfs(None::<&'static str>),
        }
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.clamp_all();
        self.set_all()
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        let mut copy = self.clone();
        copy.state.is_resuming = true;
        copy.set_all()
    }
}

impl crate::settings::OnPowerEvent for Gpu {}

impl TGpu for Gpu {
    fn limits(&self) -> crate::api::GpuLimits {
        crate::api::GpuLimits {
            fast_ppt_limits: Some(RangeLimit {
                min: super::util::range_min_or_fallback(&self.limits.fast_ppt, MIN_FAST_PPT) / self.limits.ppt_divisor.unwrap_or(PPT_DIVISOR),
                max: super::util::range_max_or_fallback(&self.limits.fast_ppt, MAX_FAST_PPT) / self.limits.ppt_divisor.unwrap_or(PPT_DIVISOR),
            }),
            slow_ppt_limits: Some(RangeLimit {
                min: super::util::range_min_or_fallback(&self.limits.slow_ppt, MIN_SLOW_PPT) / self.limits.ppt_divisor.unwrap_or(PPT_DIVISOR),
                max: super::util::range_max_or_fallback(&self.limits.slow_ppt, MIN_SLOW_PPT) / self.limits.ppt_divisor.unwrap_or(PPT_DIVISOR),
            }),
            ppt_step: self.limits.ppt_step.unwrap_or(1),
            tdp_limits: None,
            tdp_boost_limits: None,
            tdp_step: 42,
            clock_min_limits: Some(RangeLimit {
                min: super::util::range_min_or_fallback(&self.limits.clock_min, MIN_CLOCK),
                max: super::util::range_max_or_fallback(&self.limits.clock_min, MAX_CLOCK),
            }),
            clock_max_limits: Some(RangeLimit {
                min: super::util::range_min_or_fallback(&self.limits.clock_max, MIN_CLOCK),
                max: super::util::range_max_or_fallback(&self.limits.clock_max, MAX_CLOCK),
            }),
            clock_step: self.limits.clock_step.unwrap_or(100),
            memory_control: Some(RangeLimit {
                min: super::util::range_min_or_fallback(&self.limits.memory_clock, MIN_MEMORY_CLOCK),
                max: super::util::range_max_or_fallback(&self.limits.memory_clock, MAX_MEMORY_CLOCK),
            }),
            memory_step: self.limits.memory_clock_step.unwrap_or(400),
        }
    }

    fn json(&self) -> crate::persist::GpuJson {
        self.clone().into()
    }

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>) {
        self.fast_ppt = fast.map(|x| x * self.limits.ppt_divisor.unwrap_or(PPT_DIVISOR));
        self.slow_ppt = slow.map(|x| x * self.limits.ppt_divisor.unwrap_or(PPT_DIVISOR));
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        (
            self.fast_ppt.map(|x| x / self.limits.ppt_divisor.unwrap_or(PPT_DIVISOR)),
            self.slow_ppt.map(|x| x / self.limits.ppt_divisor.unwrap_or(PPT_DIVISOR)),
        )
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        self.clock_limits = limits;
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.clock_limits.as_ref()
    }

    fn memory_clock(&mut self, speed: Option<u64>) {
        self.memory_clock = speed;
    }

    fn get_memory_clock(&self) -> Option<u64> {
        self.memory_clock
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::SteamDeck
    }
}

fn parse_pp_dpm_fclk(s: &str) -> Vec<(usize, usize)> { // (value, MHz)
    let mut result = Vec::new();
    for line in s.split('\n') {
        if !line.is_empty() {
            if let Some((val, freq_mess)) = line.split_once(':') {
                if let Ok(val) = val.parse::<usize>() {
                    if let Some((freq, _unit)) = freq_mess.trim().split_once(|c: char| !c.is_digit(10)) {
                        if let Ok(freq) = freq.parse::<usize>() {
                            result.push((val, freq));
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    result
}
