use serde::{Deserialize, Serialize};
use super::RangeLimit;

#[derive(Serialize, Deserialize, Debug, Clone)]
//#[serde(tag = "target")]
pub enum GpuLimitType {
    #[serde(rename = "GabeBoy", alias = "SteamDeck")]
    SteamDeck,
    #[serde(rename = "GabeBoyAdvance", alias = "SteamDeckAdvance")]
    SteamDeckAdvance,
    #[serde(rename = "GabeBoySP", alias = "SteamDeckOLED")]
    SteamDeckOLED,
    Generic,
    GenericAMD,
    Unknown,
    DevMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericGpuLimit {
    pub fast_ppt: Option<RangeLimit<u64>>,
    pub fast_ppt_default: Option<u64>,
    pub slow_ppt: Option<RangeLimit<u64>>,
    pub slow_ppt_default: Option<u64>,
    pub ppt_divisor: Option<u64>,
    pub ppt_step: Option<u64>,
    pub tdp: Option<RangeLimit<u64>>,
    pub tdp_boost: Option<RangeLimit<u64>>,
    pub tdp_divisor: Option<u64>,
    pub tdp_step: Option<u64>,
    pub clock_min: Option<RangeLimit<u64>>,
    pub clock_max: Option<RangeLimit<u64>>,
    pub clock_step: Option<u64>,
    pub memory_clock: Option<RangeLimit<u64>>,
    pub memory_clock_step: Option<u64>,
    pub skip_resume_reclock: bool,
    pub experiments: bool,
}

impl GenericGpuLimit {
    pub fn default_for(t: GpuLimitType) -> Self {
        match t {
            GpuLimitType::SteamDeck | GpuLimitType::SteamDeckAdvance => Self::default_steam_deck(),
            GpuLimitType::SteamDeckOLED => Self::default_steam_deck_oled(),
            GpuLimitType::DevMode => Self::default_dev_mode(),
            _t => Self::default(),
        }
    }

    fn default_steam_deck() -> Self {
        Self {
            fast_ppt: Some(RangeLimit {
                min: Some(1000000),
                max: Some(30_000_000),
            }),
            fast_ppt_default: Some(15_000_000),
            slow_ppt: Some(RangeLimit {
                min: Some(1000000),
                max: Some(29_000_000),
            }),
            slow_ppt_default: Some(15_000_000),
            ppt_divisor: Some(1_000_000),
            ppt_step: Some(1),
            tdp: None,
            tdp_boost: None,
            tdp_divisor: None,
            tdp_step: None,
            clock_min: Some(RangeLimit {
                min: Some(400),
                max: Some(1600),
            }),
            clock_max: Some(RangeLimit {
                min: Some(400),
                max: Some(1600),
            }),
            clock_step: Some(100),
            // Disabled for now since LCD version is a bit broken on sysfs right now
            /*memory_clock: Some(RangeLimit {
                min: Some(400),
                max: Some(800),
            }),
            memory_clock_step: Some(400),*/
            memory_clock: None,
            memory_clock_step: None,
            skip_resume_reclock: false,
            experiments: false,
        }
    }

    fn default_steam_deck_oled() -> Self {
        let mut sd = Self::default_steam_deck();
        sd.memory_clock_step = Some(200);
        sd
    }

    fn default_dev_mode() -> Self {
        Self {
            fast_ppt: Some(RangeLimit {
                min: Some(3_000_000),
                max: Some(11_000_000),
            }),
            fast_ppt_default: Some(10_000_000),
            slow_ppt: Some(RangeLimit {
                min: Some(7_000_000),
                max: Some(11_000_000),
            }),
            slow_ppt_default: Some(10_000_000),
            ppt_divisor: Some(1_000_000),
            ppt_step: Some(1),
            tdp: Some(RangeLimit { min: Some(1_000_000), max: Some(100_000_000) }),
            tdp_boost: Some(RangeLimit { min: Some(1_000_000), max: Some(110_000_000) }),
            tdp_divisor: Some(1_000_000),
            tdp_step: Some(1),
            clock_min: Some(RangeLimit {
                min: Some(100),
                max: Some(1000),
            }),
            clock_max: Some(RangeLimit {
                min: Some(100),
                max: Some(1100),
            }),
            clock_step: Some(100),
            memory_clock: Some(RangeLimit {
                min: Some(100),
                max: Some(1100),
            }),
            memory_clock_step: Some(100),
            skip_resume_reclock: false,
            experiments: true,
        }
    }

    pub fn apply_override(&mut self, limit_override: Self) {
        if let Some(range) = limit_override.fast_ppt {
            if range.min.is_none() && range.max.is_none() {
                self.fast_ppt = None;
            } else {
                self.fast_ppt = Some(range);
            }
        }
        if let Some(def) = limit_override.fast_ppt_default {
            self.fast_ppt_default = Some(def);
        }
        if let Some(range) = limit_override.slow_ppt {
            if range.min.is_none() && range.max.is_none() {
                self.slow_ppt = None;
            } else {
                self.slow_ppt = Some(range);
            }
        }
        if let Some(def) = limit_override.slow_ppt_default {
            self.slow_ppt_default = Some(def);
        }
        if let Some(val) = limit_override.ppt_divisor {
            self.ppt_divisor = Some(val);
        }
        if let Some(val) = limit_override.ppt_step {
            self.ppt_step = Some(val);
        }
        if let Some(range) = limit_override.tdp {
            if range.min.is_none() && range.max.is_none() {
                self.tdp = None;
            } else {
                self.tdp = Some(range);
            }
        }
        if let Some(range) = limit_override.tdp_boost {
            if range.min.is_none() && range.max.is_none() {
                self.tdp_boost = None;
            } else {
                self.tdp_boost = Some(range);
            }
        }
        if let Some(val) = limit_override.tdp_step {
            self.tdp_step = Some(val);
        }
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
        self.skip_resume_reclock = limit_override.skip_resume_reclock;
        self.experiments = limit_override.experiments;
    }
}
