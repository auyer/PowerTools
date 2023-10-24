use std::convert::Into;

use sysfuss::{BasicEntityPath, SysEntity, SysEntityAttributesExt};

use limits_core::json_v2::{GenericCpusLimit, GenericCpuLimit};

use super::POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT;
use super::util::{range_max_or_fallback, range_min_or_fallback};
use crate::api::RangeLimit;
use crate::persist::CpuJson;
use crate::settings::{min_max_from_json, MinMax};
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{TCpu, TCpus, ProviderBuilder};

const CPU_PRESENT_PATH: &str = "/sys/devices/system/cpu/present";
const CPU_SMT_PATH: &str = "/sys/devices/system/cpu/smt/control";

const CARD_EXTENSIONS: &[&'static str] = &[
    super::DPM_FORCE_LIMITS_ATTRIBUTE
];

const MAX_CLOCK: u64 = 3500;
const MIN_MAX_CLOCK: u64 = 200; // minimum value allowed for maximum CPU clock, MHz
const MIN_MIN_CLOCK: u64 = 1400; // minimum value allowed for minimum CPU clock, MHz
const CLOCK_STEP: u64 = 100;

#[derive(Debug, Clone)]
pub struct Cpus {
    pub cpus: Vec<Cpu>,
    pub smt: bool,
    pub smt_capable: bool,
    pub(super) limits: GenericCpusLimit,
}

impl OnSet for Cpus {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        if self.smt_capable {
            // toggle SMT
            if self.smt {
                usdpl_back::api::files::write_single(CPU_SMT_PATH, "on")
                    .map_err(|e| SettingError {
                        msg: format!("Failed to write `on` to `{}`: {}", CPU_SMT_PATH, e),
                        setting: crate::settings::SettingVariant::Cpu,
                    })
                    .unwrap_or_else(|e| errors.push(e));
            } else {
                usdpl_back::api::files::write_single(CPU_SMT_PATH, "off")
                    .map_err(|e| SettingError {
                        msg: format!("Failed to write `off` to `{}`: {}", CPU_SMT_PATH, e),
                        setting: crate::settings::SettingVariant::Cpu,
                    })
                    .unwrap_or_else(|e| errors.push(e));
            }
        }
        for (i, cpu) in self.cpus.as_mut_slice().iter_mut().enumerate() {
            cpu.state.do_set_online = self.smt || i % 2 == 0;
            cpu.on_set().unwrap_or_else(|mut e| errors.append(&mut e));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl OnResume for Cpus {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        for cpu in &self.cpus {
            cpu.on_resume()
                .unwrap_or_else(|mut e| errors.append(&mut e));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Cpus {
    pub fn cpu_count() -> Option<usize> {
        let mut data: String = usdpl_back::api::files::read_single(CPU_PRESENT_PATH)
            .unwrap_or_else(|_| "0-7".to_string() /* Steam Deck's default */);
        if let Some(dash_index) = data.find('-') {
            let data = data.split_off(dash_index + 1);
            if let Ok(max_cpu) = data.parse::<usize>() {
                return Some(max_cpu + 1);
            }
        }
        log::warn!("Failed to parse CPU info from kernel, is Tux evil?");
        None
    }

    fn system_smt_capabilities() -> (bool, bool) {
        match usdpl_back::api::files::read_single::<_, String, _>(CPU_SMT_PATH) {
            Ok(val) => (val.trim().to_lowercase() == "on", true),
            Err(_) => (false, false),
        }
    }
}

impl ProviderBuilder<Vec<CpuJson>, GenericCpusLimit> for Cpus {
    fn from_json_and_limits(mut persistent: Vec<CpuJson>, version: u64, limits: GenericCpusLimit) -> Self {
        POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.reset();
        let (_, can_smt) = Self::system_smt_capabilities();
        let mut result = Vec::with_capacity(persistent.len());
        let max_cpus = Self::cpu_count();
        let smt_guess = crate::settings::util::guess_smt(&persistent) && can_smt;
        for (i, cpu) in persistent.drain(..).enumerate() {
            // prevent having more CPUs than available
            if let Some(max_cpus) = max_cpus {
                if i == max_cpus {
                    break;
                }
            }
            let new_cpu = if let Some(cpu_limit) = limits.cpus.get(i) {
                Cpu::from_json_and_limits(
                    cpu,
                    version,
                    i,
                    cpu_limit.to_owned()
                )
            } else {
                Cpu::from_json(
                    cpu,
                    version,
                    i,
                )
            };
            result.push(new_cpu);
        }
        if let Some(max_cpus) = max_cpus {
            if result.len() != max_cpus {
                for i in result.len()..max_cpus {
                    result.push(Cpu::system_default(i));
                }
            }
        }
        Self {
            cpus: result,
            smt: smt_guess,
            smt_capable: can_smt,
            limits: limits,
        }
    }

    fn from_limits(limits: GenericCpusLimit) -> Self {
        POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.reset();
        if let Some(max_cpu) = Self::cpu_count() {
            let mut sys_cpus = Vec::with_capacity(max_cpu);
            for i in 0..max_cpu {
                let new_cpu = if let Some(cpu_limit) = limits.cpus.get(i) {
                    Cpu::from_limits(
                        i,
                        cpu_limit.to_owned()
                    )
                } else {
                    Cpu::system_default(i)
                };
                sys_cpus.push(new_cpu);
            }
            let (_, can_smt) = Self::system_smt_capabilities();
            Self {
                cpus: sys_cpus,
                smt: true,
                smt_capable: can_smt,
                limits: limits,
            }
        } else {
            Self {
                cpus: vec![],
                smt: false,
                smt_capable: false,
                limits: limits,
            }
        }
    }
}

impl crate::settings::OnPowerEvent for Cpus {}

impl TCpus for Cpus {
    fn limits(&self) -> crate::api::CpusLimits {
        crate::api::CpusLimits {
            cpus: self.cpus.iter().map(|x| x.limits()).collect(),
            count: self.cpus.len(),
            smt_capable: self.smt_capable,
            governors: if self.limits.global_governors {
                self.cpus
                    .iter()
                    .next()
                    .map(|x| x.governors())
                    .unwrap_or_else(|| Vec::with_capacity(0))
            } else {
                Vec::with_capacity(0)
            },
        }
    }

    fn json(&self) -> Vec<crate::persist::CpuJson> {
        self.cpus.iter().map(|x| x.to_owned().into()).collect()
    }

    fn cpus(&mut self) -> Vec<&mut dyn TCpu> {
        self.cpus.iter_mut().map(|x| x as &mut dyn TCpu).collect()
    }

    fn len(&self) -> usize {
        self.cpus.len()
    }

    fn smt(&mut self) -> &'_ mut bool {
        log::debug!("CPU driver thinks SMT is {}", self.smt);
        &mut self.smt
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::SteamDeck
    }
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub online: bool,
    pub clock_limits: Option<MinMax<u64>>,
    pub governor: String,
    limits: GenericCpuLimit,
    index: usize,
    state: crate::state::steam_deck::Cpu,
    sysfs: BasicEntityPath,
}

//const CPU_CLOCK_LIMITS_PATH: &str = "/sys/class/drm/card0/device/pp_od_clk_voltage";
const CPU_CLOCK_LIMITS_ATTRIBUTE: &str = "device/pp_od_clk_voltage";

enum ClockType {
    Min = 0,
    Max = 1,
}

impl Cpu {
    #[inline]
    fn from_json_and_limits(other: CpuJson, version: u64, i: usize, oc_limits: GenericCpuLimit) -> Self {
        match version {
            0 => Self {
                online: other.online,
                clock_limits: other.clock_limits.map(|x| min_max_from_json(x, version)),
                governor: other.governor,
                limits: oc_limits,
                index: i,
                state: crate::state::steam_deck::Cpu::default(),
                sysfs: Self::find_card_sysfs(other.root),
            },
            _ => Self {
                online: other.online,
                clock_limits: other.clock_limits.map(|x| min_max_from_json(x, version)),
                governor: other.governor,
                limits: oc_limits,
                index: i,
                state: crate::state::steam_deck::Cpu::default(),
                sysfs: Self::find_card_sysfs(other.root),
            },
        }
    }

    #[inline]
    fn from_json(other: CpuJson, version: u64, i: usize) -> Self {
        let oc_limits = GenericCpuLimit::default_for(&limits_core::json_v2::CpuLimitType::SteamDeck, i);
        Self::from_json_and_limits(other, version, i, oc_limits)
    }

    fn find_card_sysfs(root: Option<impl AsRef<std::path::Path>>) -> BasicEntityPath {
        let root = crate::settings::util::root_or_default_sysfs(root);
        match root.class("drm", sysfuss::capability::attributes(crate::settings::util::CARD_NEEDS.into_iter().map(|s| s.to_string()))) {
            Ok(iter) => {
                let card = iter
                    .filter(|ent| if let Ok(name) = ent.name() { name.starts_with("card")} else { false })
                    .filter(|ent| super::util::card_also_has(ent, CARD_EXTENSIONS))
                    .next()
                    .unwrap_or_else(|| {
                        log::error!("Failed to find SteamDeck drm in sysfs (no results), using naive fallback");
                        BasicEntityPath::new(root.as_ref().join("sys/class/drm/card0"))
                    });
                log::info!("Found SteamDeck drm in sysfs: {}", card.as_ref().display());
                card
            },
            Err(e) => {
                log::error!("Failed to find SteamDeck drm in sysfs ({}), using naive fallback", e);
                BasicEntityPath::new(root.as_ref().join("sys/class/drm/card0"))
            }
        }
    }

    fn set_clock_limit(&self, index: usize, speed: u64, mode: ClockType) -> Result<(), SettingError> {
        let payload = format!("p {} {} {}\n", index / 2, mode as u8, speed);
        self.sysfs.set(CPU_CLOCK_LIMITS_ATTRIBUTE.to_owned(), &payload).map_err(|e| {
            SettingError {
                msg: format!(
                    "Failed to write `{}` to `{}`: {}",
                    &payload, CPU_CLOCK_LIMITS_ATTRIBUTE, e
                ),
                setting: crate::settings::SettingVariant::Cpu,
            }
        })
    }

    fn reset_clock_limits(&self) -> Result<(), SettingError> {
        self.sysfs.set(CPU_CLOCK_LIMITS_ATTRIBUTE.to_owned(), "r\n").map_err(|e| {
            SettingError {
                msg: format!(
                    "Failed to write `r` to `{}`: {}",
                    CPU_CLOCK_LIMITS_ATTRIBUTE, e
                ),
                setting: crate::settings::SettingVariant::Cpu,
            }
        })
    }

    fn set_clock_limits(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        if let Some(clock_limits) = &self.clock_limits {
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_cpu(true, self.index);
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.enforce_level(&self.sysfs)?;
            log::debug!(
                "Setting CPU {} (min, max) clockspeed to ({:?}, {:?})",
                self.index,
                clock_limits.min,
                clock_limits.max
            );
            self.state.clock_limits_set = true;
            // max clock
            if let Some(max) = clock_limits.max {
                self.set_clock_limit(self.index, max, ClockType::Max)
                    .unwrap_or_else(|e| errors.push(e));
            }
            // min clock
            if let Some(min) = clock_limits.min {
                let valid_min = if min < range_min_or_fallback(&self.limits.clock_min, MIN_MIN_CLOCK) {
                    range_min_or_fallback(&self.limits.clock_min, MIN_MIN_CLOCK)
                } else {
                    min
                };
                self.set_clock_limit(self.index, valid_min, ClockType::Min)
                    .unwrap_or_else(|e| errors.push(e));
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        } else if self.state.clock_limits_set
            || (self.state.is_resuming && !self.limits.skip_resume_reclock)
            || POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.needs_manual()
        {
            let mut errors = Vec::new();
            self.state.clock_limits_set = false;
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_cpu(false, self.index);
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT
                    .enforce_level(&self.sysfs)?;
            if POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.needs_manual() {
                // always set clock speeds, since it doesn't reset correctly (kernel/hardware bug)
                POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.enforce_level(&self.sysfs)?;
                // disable manual clock limits
                log::debug!("Setting CPU {} to default clockspeed", self.index);
                // max clock
                self.set_clock_limit(self.index, range_max_or_fallback(&self.limits.clock_max, MAX_CLOCK), ClockType::Max)
                    .unwrap_or_else(|e| errors.push(e));
                // min clock
                self.set_clock_limit(self.index, range_min_or_fallback(&self.limits.clock_min, MIN_MIN_CLOCK), ClockType::Min)
                    .unwrap_or_else(|e| errors.push(e));
            }
            // TODO remove this when it's no longer needed
            self.clock_unset_workaround().unwrap_or_else(|mut e| errors.append(&mut e));
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        } else {
            Ok(())
        }
    }

    // https://github.com/NGnius/PowerTools/issues/107
    fn clock_unset_workaround(&self) -> Result<(), Vec<SettingError>> {
        if !self.state.is_resuming {
            let mut errors = Vec::new();
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_cpu(true, self.index);
            // always set clock speeds, since it doesn't reset correctly (kernel/hardware bug)
            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.enforce_level(&self.sysfs)?;
            // disable manual clock limits
            log::debug!("Setting CPU {} to default clockspeed", self.index);
            // max clock
            self.set_clock_limit(self.index, range_max_or_fallback(&self.limits.clock_max, MAX_CLOCK), ClockType::Max)
                .unwrap_or_else(|e| errors.push(e));
            // min clock
            self.set_clock_limit(self.index, range_min_or_fallback(&self.limits.clock_min, MIN_MIN_CLOCK), ClockType::Min)
                .unwrap_or_else(|e| errors.push(e));

            self.set_confirm().unwrap_or_else(|e| errors.push(e));

            self.reset_clock_limits().unwrap_or_else(|e| errors.push(e));
            self.set_confirm().unwrap_or_else(|e| errors.push(e));

            POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.set_cpu(false, self.index);
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        } else {
            Ok(())
        }
    }

    fn set_confirm(&self) -> Result<(), SettingError> {
        self.sysfs.set(CPU_CLOCK_LIMITS_ATTRIBUTE.to_owned(), "c\n").map_err(|e| {
            SettingError {
                msg: format!("Failed to write `c` to `{}`: {}", CPU_CLOCK_LIMITS_ATTRIBUTE, e),
                setting: crate::settings::SettingVariant::Cpu,
            }
        })
    }

    fn set_force_performance_related(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        // set clock limits
        //log::debug!("Setting {} to manual", CPU_FORCE_LIMITS_PATH);
        //let mode: String = usdpl_back::api::files::read_single(CPU_FORCE_LIMITS_PATH.to_owned()).unwrap();
        self.set_clock_limits()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        // commit changes (if no errors have already occured)
        if errors.is_empty() {
            if POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT.needs_manual() {
                self.set_confirm().map_err(|e| vec![e])
            } else {
                Ok(())
            }
        } else {
            Err(errors)
        }
    }

    fn set_governor(&self) -> Result<(), SettingError> {
        if self.index == 0 || self.online {
            let governor_path = cpu_governor_path(self.index);
            usdpl_back::api::files::write_single(&governor_path, &self.governor).map_err(|e| {
                SettingError {
                    msg: format!(
                        "Failed to write `{}` to `{}`: {}",
                        &self.governor, &governor_path, e
                    ),
                    setting: crate::settings::SettingVariant::Cpu,
                }
            })
        } else {
            Ok(())
        }
    }

    fn set_all(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();
        // set cpu online/offline
        if self.index != 0 && self.state.do_set_online {
            // cpu0 cannot be disabled
            let online_path = cpu_online_path(self.index);
            usdpl_back::api::files::write_single(&online_path, self.online as u8)
                .map_err(|e| SettingError {
                    msg: format!("Failed to write to `{}`: {}", &online_path, e),
                    setting: crate::settings::SettingVariant::Cpu,
                })
                .unwrap_or_else(|e| errors.push(e));
        }

        self.set_force_performance_related()
            .unwrap_or_else(|mut e| errors.append(&mut e));

        self.set_governor().unwrap_or_else(|e| errors.push(e));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn clamp_all(&mut self) {
        if let Some(clock_limits) = &mut self.clock_limits {
            if let Some(min) = clock_limits.min {
                clock_limits.min =
                    Some(min.clamp(range_min_or_fallback(&self.limits.clock_max, MIN_MAX_CLOCK), range_max_or_fallback(&self.limits.clock_min, MAX_CLOCK)));
            }
            if let Some(max) = clock_limits.max {
                clock_limits.max =
                    Some(max.clamp(range_min_or_fallback(&self.limits.clock_max, MIN_MAX_CLOCK), range_max_or_fallback(&self.limits.clock_max, MAX_CLOCK)));
            }
        }
    }

    /*fn from_sys(cpu_index: usize, oc_limits: CpuLimits) -> Self {
        Self {
            online: usdpl_back::api::files::read_single(cpu_online_path(cpu_index)).unwrap_or(1u8) != 0,
            clock_limits: None,
            governor: usdpl_back::api::files::read_single(cpu_governor_path(cpu_index))
                .unwrap_or("schedutil".to_owned()),
            limits: oc_limits,
            index: cpu_index,
            state: crate::state::steam_deck::Cpu::default(),
        }
    }*/

    fn from_limits(cpu_index: usize, oc_limits: GenericCpuLimit) -> Self {
        Self {
            online: true,
            clock_limits: None,
            governor: "schedutil".to_owned(),
            limits: oc_limits,
            index: cpu_index,
            state: crate::state::steam_deck::Cpu::default(),
            sysfs: Self::find_card_sysfs(None::<&'static str>)
        }
    }

    fn system_default(cpu_index: usize) -> Self {
        Self::from_limits(cpu_index, GenericCpuLimit::default_for(&limits_core::json_v2::CpuLimitType::SteamDeck, cpu_index))
    }

    fn limits(&self) -> crate::api::CpuLimits {
        crate::api::CpuLimits {
            clock_min_limits: Some(RangeLimit {
                min: range_min_or_fallback(&self.limits.clock_max, MIN_MAX_CLOCK), // allows min to be set by max (it's weird, blame the kernel)
                max: range_max_or_fallback(&self.limits.clock_min, MAX_CLOCK),
            }),
            clock_max_limits: Some(RangeLimit {
                min: range_min_or_fallback(&self.limits.clock_max, MIN_MAX_CLOCK),
                max: range_max_or_fallback(&self.limits.clock_max, MAX_CLOCK),
            }),
            clock_step: self.limits.clock_step.unwrap_or(CLOCK_STEP),
            governors: self.governors(),
        }
    }

    fn governors(&self) -> Vec<String> {
        // NOTE: this eats errors
        let gov_str: String =
            match usdpl_back::api::files::read_single(cpu_available_governors_path(self.index)) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Error getting available CPU governors: {}", e);
                    return vec![];
                }
            };
        gov_str.split(' ').map(|s| s.to_owned()).collect()
    }
}

impl Into<CpuJson> for Cpu {
    #[inline]
    fn into(self) -> CpuJson {
        CpuJson {
            online: self.online,
            clock_limits: self.clock_limits.map(|x| x.into()),
            governor: self.governor,
            root: self.sysfs.root().and_then(|p| p.as_ref().to_str().map(|r| r.to_owned()))
        }
    }
}

impl OnSet for Cpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.clamp_all();
        self.set_all()
    }
}

impl OnResume for Cpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        let mut copy = self.clone();
        copy.state.is_resuming = true;
        copy.set_all()
    }
}

impl TCpu for Cpu {
    fn online(&mut self) -> &mut bool {
        &mut self.online
    }

    fn governor(&mut self, governor: String) {
        self.governor = governor;
    }

    fn get_governor(&self) -> &'_ str {
        &self.governor
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        self.clock_limits = limits;
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.clock_limits.as_ref()
    }
}

#[inline]
fn cpu_online_path(index: usize) -> String {
    format!("/sys/devices/system/cpu/cpu{}/online", index)
}

#[inline]
fn cpu_governor_path(index: usize) -> String {
    format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        index
    )
}

#[inline]
fn cpu_available_governors_path(index: usize) -> String {
    format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_available_governors",
        index
    )
}
