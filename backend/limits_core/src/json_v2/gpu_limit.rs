use serde::{Deserialize, Serialize};
use super::RangeLimit;

#[derive(Serialize, Deserialize, Debug, Clone)]
//#[serde(tag = "target")]
pub enum GpuLimitType {
    #[serde(rename = "GabeBoy", alias = "SteamDeck")]
    SteamDeck,
    #[serde(rename = "GabeBoyAdvance", alias = "SteamDeckAdvance")]
    SteamDeckAdvance,
    Generic,
    GenericAMD,
    Unknown,
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
    pub tdp_step: Option<u64>,
    pub clock_min: Option<RangeLimit<u64>>,
    pub clock_max: Option<RangeLimit<u64>>,
    pub clock_step: Option<u64>,
    pub skip_resume_reclock: bool,
}

impl GenericGpuLimit {
    pub fn default_for(t: GpuLimitType) -> Self {
        match t {
            GpuLimitType::SteamDeck | GpuLimitType::SteamDeckAdvance => Self::default_steam_deck(),
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
            skip_resume_reclock: false,
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
    }
}
