use std::convert::Into;

use limits_core::json_v2::GenericBatteryLimit;
use sysfuss::{SysEntity, SysEntityAttributesExt};

use crate::persist::BatteryJson;
use crate::settings::{TBattery, ProviderBuilder};
use crate::settings::{OnResume, OnSet, SettingError};

#[derive(Debug, Clone)]
pub struct Battery {
    #[allow(dead_code)]
    limits: GenericBatteryLimit,
    sysfs: sysfuss::PowerSupplyPath,
}

impl Into<BatteryJson> for Battery {
    #[inline]
    fn into(self) -> BatteryJson {
        BatteryJson {
            charge_rate: None,
            charge_mode: None,
            events: Vec::default(),
            root: self.sysfs.root().and_then(|p| p.as_ref().to_str().map(|s| s.to_owned())),
        }
    }
}

impl Battery {
    /*fn read_f64<P: AsRef<std::path::Path>>(path: P) -> Result<f64, SettingError> {
        let path = path.as_ref();
        match usdpl_back::api::files::read_single::<_, f64, _>(path) {
            Err(e) => Err(SettingError {
                msg: format!("Failed to read from `{}`: {}", path.display(), e),
                setting: crate::settings::SettingVariant::Battery,
            }),
            // this value is in uA, while it's set in mA
            // so convert this to mA for consistency
            Ok(val) => Ok(val / 1000.0),
        }
    }*/

    fn find_psu_sysfs(root: Option<impl AsRef<std::path::Path>>) -> sysfuss::PowerSupplyPath {
        let root = crate::settings::util::root_or_default_sysfs(root);
        match root.power_supply(crate::settings::util::always_satisfied) {
            Ok(iter) => {
                iter.filter(|x| x.name().is_ok_and(|name| name.starts_with("BAT")))
                    .next()
                    .unwrap_or_else(|| {
                        log::error!("Failed to find generic battery power_supply in sysfs (no results), using naive fallback");
                        root.power_supply_by_name("BAT0")
                    })
            },
            Err(e) => {
                log::error!("Failed to find generic battery power_supply in sysfs ({}), using naive fallback", e);
                root.power_supply_by_name("BAT0")
            }
        }
    }

    fn get_design_voltage(&self) -> Option<f64> {
        match self.sysfs.attribute::<f64, _>(sysfuss::PowerSupplyAttribute::VoltageMax) {
            Ok(x) => Some(x/1000000.0),
            Err(e) => {
                log::debug!("get_design_voltage voltage_max err: {}", e);
                match sysfuss::SysEntityRawExt::attribute::<_, f64, _>(&self.sysfs, "voltage_min_design".to_owned()) { // Framework 13 AMD
                    Ok(x) => Some(x/1000000.0),
                    Err(e) => {
                        log::debug!("get_design_voltage voltage_min_design err: {}", e);
                        match self.sysfs.attribute::<f64, _>(sysfuss::PowerSupplyAttribute::VoltageMin) {
                            Ok(x) => Some(x/1000000.0),
                            Err(e) => {
                                log::debug!("get_design_voltage voltage_min err: {}", e);
                                None
                            }
                        }
                    }
                }
            }
        }
    }
}

impl ProviderBuilder<BatteryJson, GenericBatteryLimit> for Battery {
    fn from_json_and_limits(
        persistent: BatteryJson,
        _version: u64,
        limits: GenericBatteryLimit,
    ) -> Self {
        // TODO
        Self {
            limits,
            sysfs: Self::find_psu_sysfs(persistent.root)
        }
    }

    fn from_limits(limits: GenericBatteryLimit) -> Self {
        // TODO
        Self {
            limits,
            sysfs: Self::find_psu_sysfs(None::<&'static str>),
        }
    }
}

impl OnSet for Battery {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        // TODO
        Ok(())
    }
}

impl OnResume for Battery {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        // TODO
        Ok(())
    }
}

impl crate::settings::OnPowerEvent for Battery {}

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
        if let Some(battery_voltage) = self.get_design_voltage() {
            match self.sysfs.attribute::<f64, _>(sysfuss::PowerSupplyAttribute::ChargeFull) {
                Ok(x) => Some(x/1000000.0 * battery_voltage),
                Err(e) => {
                    log::warn!("read_charge_full err: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    fn read_charge_now(&self) -> Option<f64> {
        if let Some(battery_voltage) = self.get_design_voltage() {
            match self.sysfs.attribute::<f64, _>(sysfuss::PowerSupplyAttribute::ChargeNow) {
                Ok(x) => Some(x/1000000.0 * battery_voltage),
                Err(e) => {
                    log::warn!("read_charge_now err: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    fn read_charge_design(&self) -> Option<f64> {
        if let Some(battery_voltage) = self.get_design_voltage() {
            match self.sysfs.attribute::<f64, _>(sysfuss::PowerSupplyAttribute::ChargeFullDesign) {
                Ok(x) => Some(x/1000000.0 * battery_voltage),
                Err(e) => {
                    log::warn!("read_charge_design err: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    fn read_current_now(&self) -> Option<f64> {
        match self.sysfs.attribute::<f64, _>(sysfuss::PowerSupplyAttribute::CurrentNow) {
            Ok(x) => Some(x/1000.0), // expects mA, reads uA
            Err(e) => {
                log::warn!("read_current_now err: {}", e);
                None
            }
        }
    }

    fn read_charge_power(&self) -> Option<f64> {
        None
    }

    fn charge_limit(&mut self, _limit: Option<f64>) {}

    fn get_charge_limit(&self) -> Option<f64> {
        /*match self.sysfs.attribute::<f64, _>(sysfuss::PowerSupplyAttribute::ChargeControlLimit) {
            Ok(x) => Some(x/1000.0),
            Err(e) => {
                log::warn!("read_charge_design err: {}", e);
                None
            }
        }*/
        None
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Generic
    }
}
