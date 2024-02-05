use std::convert::Into;

use limits_core::json_v2::GenericBatteryLimit;

use crate::persist::BatteryJson;
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{ProviderBuilder, TBattery};

#[derive(Clone)]
pub struct Battery {
    persist: BatteryJson,
    version: u64,
    limits: GenericBatteryLimit,
    charge_limit: Option<f64>,
}

impl std::fmt::Debug for Battery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("dev_mode_Battery")
            //.field("persist", &self.persist)
            .field("version", &self.version)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

impl Into<BatteryJson> for Battery {
    #[inline]
    fn into(self) -> BatteryJson {
        self.persist
    }
}

impl ProviderBuilder<BatteryJson, GenericBatteryLimit> for Battery {
    fn from_json_and_limits(
        persist: BatteryJson,
        version: u64,
        limits: GenericBatteryLimit,
    ) -> Self {
        Battery {
            persist,
            version,
            limits,
            charge_limit: None,
        }
    }

    fn from_limits(limits: GenericBatteryLimit) -> Self {
        Battery {
            persist: BatteryJson {
                charge_rate: None,
                charge_mode: None,
                events: vec![],
                root: None,
            },
            version: 0,
            limits,
            charge_limit: None,
        }
    }
}

impl OnSet for Battery {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Battery::on_set(self)");
        Ok(())
    }
}

impl OnResume for Battery {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Battery::on_resume(self)");
        Ok(())
    }
}

impl crate::settings::OnPowerEvent for Battery {}

impl crate::settings::OnLoad for Battery {
    fn on_load(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Battery::on_load(self)");
        Ok(())
    }
}

impl crate::settings::OnUnload for Battery {
    fn on_unload(&mut self) -> Result<(), Vec<SettingError>> {
        log::debug!("dev_mode_Battery::on_unload(self)");
        Ok(())
    }
}

impl TBattery for Battery {
    fn limits(&self) -> crate::api::BatteryLimits {
        log::debug!("dev_mode_Battery::limits(self) -> {{...}}");
        crate::api::BatteryLimits {
            charge_current: self.limits.charge_rate.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(11),
                max: lim.max.unwrap_or(1111),
            }),
            charge_current_step: 10,
            charge_modes: self.limits.charge_modes.clone(),
            charge_limit: self.limits.charge_limit.map(|lim| crate::api::RangeLimit {
                min: lim.min.unwrap_or(2.0),
                max: lim.max.unwrap_or(98.0),
            }),
            charge_limit_step: 1.0,
        }
    }

    fn json(&self) -> crate::persist::BatteryJson {
        log::debug!("dev_mode_Battery::json(self) -> {{...}}");
        self.clone().into()
    }

    fn charge_rate(&mut self, rate: Option<u64>) {
        log::debug!("dev_mode_Battery::charge_rate(self, {:?})", rate);
        self.persist.charge_rate = rate;
    }

    fn get_charge_rate(&self) -> Option<u64> {
        log::debug!(
            "dev_mode_Battery::get_charge_rate(self) -> {:?}",
            self.persist.charge_rate
        );
        self.persist.charge_rate
    }

    fn charge_mode(&mut self, rate: Option<String>) {
        log::debug!("dev_mode_Battery::charge_mode(self, {:?})", rate);
        self.persist.charge_mode = rate;
    }

    fn get_charge_mode(&self) -> Option<String> {
        log::debug!(
            "dev_mode_Battery::get_charge_mode(self) -> {:?}",
            self.persist.charge_mode
        );
        self.persist.charge_mode.clone()
    }

    fn read_charge_full(&self) -> Option<f64> {
        log::debug!("dev_mode_Battery::read_charge_full(self) -> None");
        None
    }

    fn read_charge_now(&self) -> Option<f64> {
        log::debug!("dev_mode_Battery::read_charge_now(self) -> None");
        None
    }

    fn read_charge_design(&self) -> Option<f64> {
        log::debug!("dev_mode_Battery::read_charge_design(self) -> None");
        None
    }

    fn read_current_now(&self) -> Option<f64> {
        log::debug!("dev_mode_Battery::read_current_now(self) -> None");
        None
    }

    fn read_charge_power(&self) -> Option<f64> {
        log::debug!("dev_mode_Battery::read_charge_power(self) -> None");
        None
    }

    fn charge_limit(&mut self, limit: Option<f64>) {
        log::debug!("dev_mode_Battery::charge_limit(self, {:?})", limit);
        self.charge_limit = limit;
    }

    fn get_charge_limit(&self) -> Option<f64> {
        log::debug!(
            "dev_mode_Battery::get_charge_limit(self) -> {:?}",
            self.charge_limit
        );
        self.charge_limit
    }

    fn provider(&self) -> crate::persist::DriverJson {
        log::debug!("dev_mode_Battery::provider(self) -> DevMode");
        crate::persist::DriverJson::DevMode
    }
}
