use std::convert::Into;

use limits_core::json_v2::{GenericCpuLimit, GenericCpusLimit};

use crate::persist::CpuJson;
use crate::settings::MinMax;
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{ProviderBuilder, TCpu, TCpus};

#[derive(Debug, Clone)]
pub struct Cpus {
    cpus: Vec<Cpu>,
    #[allow(dead_code)]
    version: u64,
    smt_enabled: bool,
    #[allow(dead_code)]
    limits: GenericCpusLimit,
}

impl OnSet for Cpus {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Cpus::on_set(self)");
        for cpu in self.cpus.iter_mut() {
            cpu.on_set()?;
        }
        Ok(())
    }
}

impl OnResume for Cpus {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Cpus::on_resume(self)");
        for cpu in self.cpus.iter() {
            cpu.on_resume()?;
        }
        Ok(())
    }
}

impl crate::settings::OnPowerEvent for Cpus {}

impl crate::settings::OnLoad for Cpus {
    fn on_load(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Cpus::on_load(self)");
        Ok(())
    }
}

impl crate::settings::OnUnload for Cpus {
    fn on_unload(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Cpus::on_unload(self)");
        Ok(())
    }
}

impl ProviderBuilder<Vec<CpuJson>, GenericCpusLimit> for Cpus {
    fn from_json_and_limits(
        persistent: Vec<CpuJson>,
        version: u64,
        limits: GenericCpusLimit,
    ) -> Self {
        let mut cpus = Vec::with_capacity(persistent.len());
        for (i, cpu) in persistent.iter().enumerate() {
            cpus.push(Cpu::from_json_and_limits(
                cpu.to_owned(),
                version,
                i,
                limits.cpus.get(i).map(|x| x.to_owned()).unwrap_or_else(|| {
                    log::warn!("No cpu limit for index {}, using default", i);
                    Default::default()
                }),
            ));
        }
        let smt_guess = crate::settings::util::guess_smt(&persistent);
        Self {
            cpus,
            version,
            smt_enabled: smt_guess,
            limits,
        }
    }

    fn from_limits(limits: GenericCpusLimit) -> Self {
        let mut cpus = Vec::with_capacity(limits.cpus.len());
        for (i, cpu) in limits.cpus.iter().enumerate() {
            cpus.push(Cpu::from_limits(i, cpu.to_owned()));
        }
        Self {
            cpus,
            version: 0,
            smt_enabled: true,
            limits,
        }
    }
}

impl TCpus for Cpus {
    fn limits(&self) -> crate::api::CpusLimits {
        log::debug!("dev_mode_Cpus::limits(self) -> {{...}}");
        crate::api::CpusLimits {
            cpus: self.cpus.iter().map(|x| x.limits()).collect(),
            count: self.cpus.len(),
            smt_capable: true,
            governors: vec![
                "this".to_owned(),
                "is".to_owned(),
                "dev".to_owned(),
                "mode".to_owned(),
            ],
        }
    }

    fn json(&self) -> Vec<CpuJson> {
        log::debug!("dev_mode_Cpus::json(self) -> {{...}}");
        self.cpus.iter().map(|x| x.to_owned().into()).collect()
    }

    fn cpus(&mut self) -> Vec<&mut dyn TCpu> {
        log::debug!("dev_mode_Cpus::cpus(self) -> {{...}}");
        self.cpus.iter_mut().map(|x| x as &mut dyn TCpu).collect()
    }

    fn len(&self) -> usize {
        log::debug!("dev_mode_Cpus::len(self) -> {}", self.cpus.len());
        self.cpus.len()
    }

    fn smt(&mut self) -> &'_ mut bool {
        log::debug!("dev_mode_Cpus::smt(self) -> {}", self.smt_enabled);
        &mut self.smt_enabled
    }

    fn provider(&self) -> crate::persist::DriverJson {
        log::debug!("dev_mode_Cpus::provider(self) -> DevMode");
        crate::persist::DriverJson::DevMode
    }
}

#[derive(Clone)]
pub struct Cpu {
    persist: CpuJson,
    version: u64,
    index: usize,
    limits: GenericCpuLimit,
    clock_limits: Option<MinMax<u64>>,
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("dev_mode_Cpu")
            //.field("persist", &self.persist)
            .field("version", &self.version)
            .field("index", &self.index)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

impl Cpu {
    #[inline]
    pub fn from_json_and_limits(
        other: CpuJson,
        version: u64,
        i: usize,
        limits: GenericCpuLimit,
    ) -> Self {
        let clock_limits = other.clock_limits.clone().map(|lim| MinMax {
            min: lim.min,
            max: lim.max,
        });
        match version {
            0 => Self {
                persist: other,
                version,
                index: i,
                limits,
                clock_limits,
            },
            _ => Self {
                persist: other,
                version,
                index: i,
                limits,
                clock_limits,
            },
        }
    }

    #[inline]
    pub fn from_limits(i: usize, limits: GenericCpuLimit) -> Self {
        Self {
            persist: CpuJson {
                online: true,
                clock_limits: None,
                governor: "".to_owned(),
                root: None,
            },
            version: 0,
            index: i,
            limits,
            clock_limits: None,
        }
    }

    fn limits(&self) -> crate::api::CpuLimits {
        crate::api::CpuLimits {
            clock_min_limits: self.limits.clock_min.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(1100),
                max: lim.max.unwrap_or(6900),
            }),
            clock_max_limits: self.limits.clock_max.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(4200),
                max: lim.max.unwrap_or(4300),
            }),
            clock_step: self.limits.clock_step.unwrap_or(11),
            governors: vec![
                "this".to_owned(),
                "is".to_owned(),
                "dev".to_owned(),
                "mode".to_owned(),
            ],
        }
    }
}

impl Into<CpuJson> for Cpu {
    #[inline]
    fn into(self) -> CpuJson {
        self.persist
    }
}

impl OnSet for Cpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Cpu::on_set(self)");
        Ok(())
    }
}

impl OnResume for Cpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Cpu::on_resume(self)");
        Ok(())
    }
}

impl TCpu for Cpu {
    fn online(&mut self) -> &mut bool {
        log::debug!("dev_mode_Cpu::online(self) -> {}", self.persist.online);
        &mut self.persist.online
    }

    fn governor(&mut self, governor: String) {
        log::debug!("dev_mode_Cpu::governor(self, {})", governor);
        self.persist.governor = governor;
    }

    fn get_governor(&self) -> &'_ str {
        log::debug!("dev_mode_Cpu::governor(self) -> {}", self.persist.governor);
        &self.persist.governor
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        log::debug!("dev_mode_Cpu::clock_limits(self, {:?})", limits);
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
            "dev_mode_Cpu::get_clock_limits(self) -> {:?}",
            self.clock_limits.as_ref()
        );
        self.clock_limits.as_ref()
    }
}
