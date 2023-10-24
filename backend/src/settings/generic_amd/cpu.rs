use crate::persist::CpuJson;
use crate::settings::generic::{Cpu as GenericCpu, Cpus as GenericCpus, FromGenericCpuInfo};
use crate::settings::MinMax;
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{TCpu, TCpus, ProviderBuilder};

#[derive(Debug)]
pub struct Cpus {
    generic: GenericCpus<Cpu>,
}

impl ProviderBuilder<Vec<CpuJson>, limits_core::json_v2::GenericCpusLimit> for Cpus {
    fn from_limits(limits: limits_core::json_v2::GenericCpusLimit) -> Self {
        Self {
            generic: GenericCpus::from_limits(limits),
        }
    }

    fn from_json_and_limits(
        other: Vec<CpuJson>,
        version: u64,
        limits: limits_core::json_v2::GenericCpusLimit,
    ) -> Self {
        Self {
            generic: GenericCpus::from_json_and_limits(other, version, limits),
        }
    }
}

impl OnResume for Cpus {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        self.generic.on_resume()
        // TODO
    }
}

impl OnSet for Cpus {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.generic.on_set()
        // TODO
    }
}

impl crate::settings::OnPowerEvent for Cpus {}

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

impl FromGenericCpuInfo for Cpu {
    fn from_limits(cpu_index: usize, limits: limits_core::json_v2::GenericCpuLimit) -> Self {
        let gen = GenericCpu::from_limits(cpu_index, limits.clone());
        Self { generic: gen }
    }

    fn from_json_and_limits(
        other: CpuJson,
        version: u64,
        cpu_index: usize,
        limits: limits_core::json_v2::GenericCpuLimit,
    ) -> Self {
        let gen = GenericCpu::from_json_and_limits(other, version, cpu_index, limits);
        Self { generic: gen }
    }
}

impl AsRef<GenericCpu> for Cpu {
    fn as_ref(&self) -> &GenericCpu {
        &self.generic
    }
}

impl AsMut<GenericCpu> for Cpu {
    fn as_mut(&mut self) -> &mut GenericCpu {
        &mut self.generic
    }
}

impl OnResume for Cpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        self.generic.on_resume()
        // TODO
    }
}

impl OnSet for Cpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.generic.on_set()
        // TODO
    }
}

impl crate::settings::OnPowerEvent for Cpu {}

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

    fn clock_limits(&mut self, _limits: Option<MinMax<u64>>) {
        //self.generic.clock_limits(limits)
        // TODO: support this
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.generic.get_clock_limits()
    }
}
