use serde::{Deserialize, Serialize};

use super::RangeLimit;

#[derive(Serialize, Deserialize, Debug, Clone)]
//#[serde(tag = "target")]
pub enum CpuLimitType {
    #[serde(rename = "GabeBoy", alias = "SteamDeck")]
    SteamDeck,
    #[serde(rename = "GabeBoyAdvance", alias = "SteamDeckAdvance")]
    SteamDeckAdvance,
    Generic,
    GenericAMD,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericCpusLimit {
    pub cpus: Vec<GenericCpuLimit>,
    pub global_governors: bool,
}

impl GenericCpusLimit {
    pub fn default_for(t: CpuLimitType) -> Self {
        match t {
            CpuLimitType::SteamDeck | CpuLimitType::SteamDeckAdvance => {
                Self {
                    cpus: [(); 8].iter().enumerate().map(|(i, _)| GenericCpuLimit::default_for(&t, i)).collect(),
                    global_governors: true,
                }
            },
            t => {
                let cpu_count = Self::cpu_count().unwrap_or(8);
                let mut cpus = Vec::with_capacity(cpu_count);
                for i in 0..cpu_count {
                    cpus.push(GenericCpuLimit::default_for(&t, i));
                }
                Self {
                    cpus,
                    global_governors: true,
                }
            }
        }
    }

    fn cpu_count() -> Option<usize> {
        let mut data: String = std::fs::read_to_string("/sys/devices/system/cpu/present")
            .unwrap_or_else(|_| "0-7".to_string() /* Steam Deck's default */);
        if let Some(dash_index) = data.find('-') {
            let data = data.split_off(dash_index + 1);
            if let Ok(max_cpu) = data.parse::<usize>() {
                return Some(max_cpu + 1);
            }
        }
        None
    }

    pub fn apply_override(&mut self, limit_override: Self) {
        if self.cpus.len() != limit_override.cpus.len() && !limit_override.cpus.is_empty() {
            // assume limit_override.cpus wants to override even the cpu count
            self.cpus = limit_override.cpus;
        } else {
            self.cpus.iter_mut()
                .zip(limit_override.cpus.into_iter())
                .for_each(|(cpu, limit_override)| cpu.apply_override(limit_override));
        }
        self.global_governors = limit_override.global_governors;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericCpuLimit {
    pub clock_min: Option<RangeLimit<u64>>,
    pub clock_max: Option<RangeLimit<u64>>,
    pub clock_step: Option<u64>,
    pub skip_resume_reclock: bool,
}

impl GenericCpuLimit {
    pub fn default_for(t: &CpuLimitType, _index: usize) -> Self {
        match t {
            CpuLimitType::SteamDeck | CpuLimitType::SteamDeckAdvance => Self::default_steam_deck(),
            _ => Self {
                clock_min: None,
                clock_max: None,
                clock_step: Some(100),
                skip_resume_reclock: false,
            },
        }
    }

    fn default_steam_deck() -> Self {
        Self {
            clock_min: Some(RangeLimit {
                min: Some(1400),
                max: Some(3500),
            }),
            clock_max: Some(RangeLimit {
                min: Some(400),
                max: Some(3500),
            }),
            clock_step: Some(100),
            skip_resume_reclock: false,
        }
    }

    pub fn apply_override(&mut self, limit_override: Self) {
        if let Some(range) = limit_override.clock_min {
            if range.min.is_none() && range.max.is_none() {
                self.clock_min = None;
            } else {
                self.clock_min = Some(range);
            }
        }
        if let Some(range) = limit_override.clock_max {
            if range.min.is_none() && range.max.is_none() {
                self.clock_max = None;
            } else {
                self.clock_max = Some(range);
            }
        }
        if let Some(val) = limit_override.clock_step {
            self.clock_step = Some(val);
        }
        self.clock_step = limit_override.clock_step;
        self.skip_resume_reclock = limit_override.skip_resume_reclock;
    }
}
