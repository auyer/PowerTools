use std::path::PathBuf;
//use std::sync::{Arc, Mutex};

//use super::{Battery, Cpus, Gpu};
use super::{OnResume, OnSet, SettingError};
use super::{TBattery, TCpus, TGeneral, TGpu};
use crate::persist::{SettingsJson, FileJson};
//use crate::utility::unwrap_lock;

const LATEST_VERSION: u64 = 0;

#[derive(Debug, Clone, Copy)]
pub enum SettingVariant {
    Battery,
    Cpu,
    Gpu,
    General,
}

impl std::fmt::Display for SettingVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Battery => write!(f, "Battery"),
            Self::Cpu => write!(f, "CPU"),
            Self::Gpu => write!(f, "GPU"),
            Self::General => write!(f, "General"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct General {
    pub persistent: bool,
    pub path: PathBuf,
    pub name: String,
    pub variant_id: u64,
    pub variant_name: String,
    pub driver: crate::persist::DriverJson,
}

impl OnSet for General {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl OnResume for General {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl crate::settings::OnPowerEvent for General {}

impl TGeneral for General {
    fn limits(&self) -> crate::api::GeneralLimits {
        crate::api::GeneralLimits {}
    }

    fn get_persistent(&self) -> bool {
        self.persistent
    }

    fn persistent(&mut self) -> &'_ mut bool {
        &mut self.persistent
    }

    fn get_path(&self) -> &'_ std::path::Path {
        &self.path
    }

    fn path(&mut self, path: std::path::PathBuf) {
        self.path = path;
    }

    fn get_name(&self) -> &'_ str {
        &self.name
    }

    fn name(&mut self, name: String) {
        self.name = name;
    }

    fn get_variant_id(&self) -> u64 {
        self.variant_id
    }

    fn variant_id(&mut self, id: u64) {
        self.variant_id = id;
    }

    fn variant_name(&mut self, name: String) {
        self.variant_name = name;
    }

    fn get_variants(&self) -> Vec<crate::api::VariantInfo> {
        if let Ok(file) = crate::persist::FileJson::open(self.get_path()) {
            file.variants.into_iter()
                .map(|(id, conf)| crate::api::VariantInfo {
                    id: id.to_string(),
                    name: conf.name,
                })
                .collect()
        } else {
            vec![self.get_variant_info()]
        }
    }

    fn add_variant(&self, variant: crate::persist::SettingsJson) -> Result<Vec<crate::api::VariantInfo>, SettingError> {
        let variant_name = variant.name.clone();
        crate::persist::FileJson::update_variant_or_create(self.get_path(), variant, variant_name)
            .map_err(|e| SettingError {
                msg: format!("failed to add variant: {}", e),
                setting: SettingVariant::General,
            })
            .map(|file| file.variants.into_iter()
                .map(|(id, conf)| crate::api::VariantInfo {
                    id: id.to_string(),
                    name: conf.name,
                })
                .collect())
    }

    fn get_variant_info(&self) -> crate::api::VariantInfo {
        crate::api::VariantInfo {
            id: self.variant_id.to_string(),
            name: self.variant_name.clone(),
        }
    }

    fn provider(&self) -> crate::persist::DriverJson {
        self.driver.clone()
    }
}

#[derive(Debug)]
pub struct Settings {
    pub general: Box<dyn TGeneral>,
    pub cpus: Box<dyn TCpus>,
    pub gpu: Box<dyn TGpu>,
    pub battery: Box<dyn TBattery>,
}

impl OnSet for Settings {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        log::debug!("Applying settings for on_set");
        self.general
            .on_set()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Set general");
        self.battery
            .on_set()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Set battery");
        self.cpus
            .on_set()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Set CPUs");
        self.gpu
            .on_set()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Set GPU");

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Settings {
    #[inline]
    pub fn from_json(name: String, other: SettingsJson, json_path: PathBuf) -> Self {
        let x = super::Driver::init(name, &other, json_path.clone());
        log::info!(
            "Loaded settings with drivers general:{:?},cpus:{:?},gpu:{:?},battery:{:?}",
            x.general.provider(),
            x.cpus.provider(),
            x.gpu.provider(),
            x.battery.provider()
        );
        Self {
            general: x.general,
            cpus: x.cpus,
            gpu: x.gpu,
            battery: x.battery,
        }
    }

    pub fn system_default(json_path: PathBuf, name: String, variant_id: u64, variant_name: String) -> Self {
        let driver = super::Driver::system_default(json_path, name, variant_id, variant_name);
        Self {
            general: driver.general,
            cpus: driver.cpus,
            gpu: driver.gpu,
            battery: driver.battery,
        }
    }

    pub fn load_system_default(&mut self, name: String, variant_id: u64, variant_name: String) {
        let driver = super::Driver::system_default(self.general.get_path().to_owned(), name, variant_id, variant_name);
        self.cpus = driver.cpus;
        self.gpu = driver.gpu;
        self.battery = driver.battery;
        self.general = driver.general;
    }

    pub fn get_variant<'a>(settings_file: &'a FileJson, variant_id: u64, variant_name: String) -> Result<&'a SettingsJson, SettingError> {
        if let Some(variant) = settings_file.variants.get(&variant_id) {
            Ok(variant)
        } else {
            Err(SettingError {
                msg: format!("Cannot get non-existent variant `{}` (id:{})", variant_name, variant_id),
                setting: SettingVariant::General,
            })
        }
    }

    pub fn load_file(
        &mut self,
        filename: PathBuf,
        name: String,
        variant: u64,
        variant_name: String,
        system_defaults: bool,
    ) -> Result<bool, SettingError> {
        let json_path = crate::utility::settings_dir().join(&filename);
        if json_path.exists() {
            if variant == u64::MAX {
                *self.general.persistent() = true;
                let file_json = FileJson::update_variant_or_create(&json_path, self.json(), variant_name.clone()).map_err(|e| SettingError {
                    msg: format!("Failed to open settings {}: {}", json_path.display(), e),
                    setting: SettingVariant::General,
                })?;
                self.general.variant_id(file_json.variants.iter().find(|(_key, val)| val.name == variant_name).map(|(key, _val)| *key).expect("Setting variant was not added properly"));
                self.general.variant_name(variant_name);
            } else {
                let file_json = FileJson::open(&json_path).map_err(|e| SettingError {
                    msg: format!("Failed to open settings {}: {}", json_path.display(), e),
                    setting: SettingVariant::General,
                })?;
                let settings_json = Self::get_variant(&file_json, variant, variant_name)?;
                if !settings_json.persistent {
                    log::warn!(
                        "Loaded persistent config `{}` ({}) with persistent=false",
                        &settings_json.name,
                        json_path.display()
                    );
                    *self.general.persistent() = false;
                    self.general.name(name);
                } else {
                    let x = super::Driver::init(name, settings_json, json_path.clone());
                    log::info!("Loaded settings with drivers general:{:?},cpus:{:?},gpu:{:?},battery:{:?}", x.general.provider(), x.cpus.provider(), x.gpu.provider(), x.battery.provider());
                    self.general = x.general;
                    self.cpus = x.cpus;
                    self.gpu = x.gpu;
                    self.battery = x.battery;
                }
            }

        } else {
            if system_defaults {
                self.load_system_default(name, variant, variant_name);
            } else {
                self.general.name(name);
                self.general.variant_name(variant_name);
            }
            *self.general.persistent() = false;
        }
        self.general.path(filename);
        self.general.variant_id(variant);
        Ok(*self.general.persistent())
    }

    /*
    pub fn load_file(&mut self, filename: PathBuf, name: String, system_defaults: bool) -> Result<bool, SettingError> {
        let json_path = crate::utility::settings_dir().join(filename);
        //let mut general_lock = unwrap_lock(self.general.lock(), "general");
        if json_path.exists() {
            let settings_json = SettingsJson::open(&json_path).map_err(|e| SettingError {
                msg: e.to_string(),
                setting: SettingVariant::General,
            })?;
            if !settings_json.persistent {
                log::warn!("Loaded persistent config `{}` ({}) with persistent=false", &settings_json.name, json_path.display());
                *self.general.persistent() = false;
                self.general.name(name);
            } else {
                self.cpus = Box::new(super::steam_deck::Cpus::from_json(settings_json.cpus, settings_json.version));
                self.gpu = Box::new(super::steam_deck::Gpu::from_json(settings_json.gpu, settings_json.version));
                self.battery = Box::new(super::steam_deck::Battery::from_json(settings_json.battery, settings_json.version));
                *self.general.persistent() = true;
                self.general.name(settings_json.name);
            }
        } else {
            if system_defaults {
                self.load_system_default();
            }
            *self.general.persistent() = false;
            self.general.name(name);
        }
        self.general.path(json_path);
        Ok(*self.general.persistent())
    }*/

    pub fn json(&self) -> SettingsJson {
        let variant_info = self.general.get_variant_info();
        SettingsJson {
            version: LATEST_VERSION,
            name: variant_info.name,
            variant: self.general.get_variant_id(),
            persistent: self.general.get_persistent(),
            cpus: self.cpus.json(),
            gpu: self.gpu.json(),
            battery: self.battery.json(),
            provider: Some(self.general.provider()),
        }
    }
}

impl OnResume for Settings {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        log::debug!("Applying settings for on_resume");
        self.general
            .on_resume()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Resumed general");
        self.battery
            .on_resume()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Resumed battery");
        self.cpus
            .on_resume()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Resumed CPUs");
        self.gpu
            .on_resume()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        log::debug!("Resumed GPU");

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl crate::settings::OnPowerEvent for Settings {
    fn on_power_event(&mut self, new_mode: super::PowerMode) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        self.general
            .on_power_event(new_mode)
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.battery
            .on_power_event(new_mode)
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.cpus
            .on_power_event(new_mode)
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.gpu
            .on_power_event(new_mode)
            .unwrap_or_else(|mut e| errors.append(&mut e));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/*impl Into<SettingsJson> for Settings {
    #[inline]
    fn into(self) -> SettingsJson {
        log::debug!("Converting into json");
        SettingsJson {
            version: LATEST_VERSION,
            name: self.general.get_name().to_owned(),
            persistent: self.general.get_persistent(),
            cpus: self.cpus.json(),
            gpu: self.gpu.json(),
            battery: self.battery.json(),
            provider: Some(self.general.provider()),
        }
    }
}*/
