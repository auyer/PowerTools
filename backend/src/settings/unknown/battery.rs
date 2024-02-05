use std::convert::Into;

use limits_core::json_v2::GenericBatteryLimit;

use crate::persist::BatteryJson;
use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::{ProviderBuilder, TBattery};

#[derive(Debug, Clone)]
pub struct Battery;

impl Battery {
    #[inline]
    fn system_default() -> Self {
        Battery
    }
}

impl Into<BatteryJson> for Battery {
    #[inline]
    fn into(self) -> BatteryJson {
        BatteryJson {
            charge_rate: None,
            charge_mode: None,
            events: Vec::default(),
            root: None,
        }
    }
}

impl ProviderBuilder<BatteryJson, GenericBatteryLimit> for Battery {
    fn from_json_and_limits(
        _persistent: BatteryJson,
        _version: u64,
        _limits: GenericBatteryLimit,
    ) -> Self {
        Battery::system_default()
    }

    fn from_limits(_limits: GenericBatteryLimit) -> Self {
        Battery::system_default()
    }
}

impl OnSet for Battery {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl OnResume for Battery {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl crate::settings::OnPowerEvent for Battery {}

impl crate::settings::OnLoad for Battery {
    fn on_load(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl crate::settings::OnUnload for Battery {
    fn on_unload(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl TBattery for Battery {
    fn limits(&self) -> crate::api::BatteryLimits {
        crate::api::BatteryLimits {
            charge_current: None,
            charge_current_step: 50,
            charge_modes: vec![],
            charge_limit: None,
            charge_limit_step: 1.0,
        }
    }

    fn json(&self) -> crate::persist::BatteryJson {
        self.clone().into()
    }

    fn charge_rate(&mut self, _rate: Option<u64>) {}

    fn get_charge_rate(&self) -> Option<u64> {
        None
    }

    fn charge_mode(&mut self, _rate: Option<String>) {}

    fn get_charge_mode(&self) -> Option<String> {
        None
    }

    fn read_charge_full(&self) -> Option<f64> {
        None
    }

    fn read_charge_now(&self) -> Option<f64> {
        None
    }

    fn read_charge_design(&self) -> Option<f64> {
        None
    }

    fn read_current_now(&self) -> Option<f64> {
        None
    }

    fn read_charge_power(&self) -> Option<f64> {
        None
    }

    fn charge_limit(&mut self, _limit: Option<f64>) {}

    fn get_charge_limit(&self) -> Option<f64> {
        None
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Unknown
    }
}
