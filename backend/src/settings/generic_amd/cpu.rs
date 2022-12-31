use crate::persist::CpuJson;
use crate::settings::MinMax;
use crate::settings::generic::{Cpu as GenericCpu, Cpus as GenericCpus};
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{TCpus, TCpu};

#[derive(Debug)]
pub struct Cpus {
    generic: GenericCpus,
}

impl Cpus {
    pub fn from_limits(limits: limits_core::json::GenericCpuLimit) -> Self {
        Self {
            generic: GenericCpus::from_limits(limits),
        }
    }

    pub fn from_json_and_limits(other: Vec<CpuJson>, version: u64, limits: limits_core::json::GenericCpuLimit) -> Self {
        Self {
            generic: GenericCpus::from_json_and_limits(other, version, limits),
        }
    }
}

impl OnResume for Cpus {
    fn on_resume(&self) -> Result<(), SettingError> {
        self.generic.on_resume()
        // TODO
    }
}

impl OnSet for Cpus {
    fn on_set(&mut self) -> Result<(), SettingError> {
        self.generic.on_set()
        // TODO
    }
}

impl TCpus for Cpus {
    fn limits(&self) -> crate::api::CpusLimits {
        self.generic.limits()
    }

    fn json(&self) -> Vec<crate::persist::CpuJson> {
        self.generic.json() // TODO
    }

    fn cpus(&mut self) -> Vec<&mut dyn TCpu> {
        self.generic.cpus() // TODO
    }

    fn len(&self) -> usize {
        self.generic.len() // TODO
    }

    fn smt(&mut self) -> &'_ mut bool {
        self.generic.smt()
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::GenericAMD
    }
}

#[derive(Debug)]
pub struct Cpu {
    generic: GenericCpu,
}

impl TCpu for Cpu {
    fn online(&mut self) -> &mut bool {
        self.generic.online()
    }

    fn governor(&mut self, governor: String) {
        self.generic.governor(governor)
    }

    fn get_governor(&self) -> &'_ str {
        self.generic.get_governor()
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        self.generic.clock_limits(limits) // TODO
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.generic.get_clock_limits() // TODO
    }
}