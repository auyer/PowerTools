use serde::{Deserialize, Serialize};
use super::RangeLimit;

#[derive(Serialize, Deserialize, Debug, Clone)]
//#[serde(tag = "target")]
pub enum BatteryLimitType {
    #[serde(rename = "GabeBoy", alias = "SteamDeck")]
    SteamDeck,
    #[serde(rename = "GabeBoySP", alias = "SteamDeckOLED")]
    SteamDeckOLED,
    Generic,
    Unknown,
    DevMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericBatteryLimit {
    pub charge_rate: Option<RangeLimit<u64>>,
    pub charge_modes: Vec<String>,
    pub charge_limit: Option<RangeLimit<f64>>, // battery charge %
    pub extra_readouts: bool,
    pub extras: super::LimitExtras,
}

impl GenericBatteryLimit {
    pub fn default_for(t: BatteryLimitType) -> Self {
        match t {
            BatteryLimitType::SteamDeck | BatteryLimitType::SteamDeckOLED => Self::default_steam_deck(),
            BatteryLimitType::DevMode => Self::default_dev_mode(),
            _t => Self::default(),
        }
    }

    fn default_steam_deck() -> Self {
        Self {
            charge_rate: Some(RangeLimit {
                min: Some(250),
                max: Some(2500),
            }),
            charge_modes: vec![
                "normal".to_owned(),
                "discharge".to_owned(),
                "idle".to_owned(),
            ],
            charge_limit: Some(RangeLimit {
                min: Some(10.0),
                max: Some(90.0),
            }),
            extra_readouts: false,
            extras: Default::default(),
        }
    }

    fn default_dev_mode() -> Self {
        Self {
            charge_rate: Some(RangeLimit {
                min: Some(0),
                max: Some(1_000),
            }),
            charge_modes: vec![
                "normal".to_owned(),
                "discharge".to_owned(),
                "idle".to_owned(),
            ],
            charge_limit: Some(RangeLimit {
                min: Some(1.0),
                max: Some(99.0),
            }),
            extra_readouts: true,
            extras: super::LimitExtras {
                experiments: true,
                quirks: vec!["".to_owned()].into_iter().collect(),
            },
        }
    }

    pub fn apply_override(&mut self, limit_override: Self) {
        if let Some(range) = limit_override.charge_rate {
            if range.min.is_none() && range.max.is_none() {
                self.charge_rate = None;
            } else {
                self.charge_rate = Some(range);
            }
        }
        if self.charge_modes.len() != limit_override.charge_modes.len() && !limit_override.charge_modes.is_empty() {
            // assume limit_override.cpus wants to override even the cpu count
            self.charge_modes = limit_override.charge_modes;
        }
        if let Some(range) = limit_override.charge_limit {
            if range.min.is_none() && range.max.is_none() {
                self.charge_limit = None;
            } else {
                self.charge_limit = Some(range);
            }
        }
        self.extra_readouts = limit_override.extra_readouts;
        self.extras = limit_override.extras;
    }
}
