use std::path::PathBuf;
//use std::sync::{Arc, Mutex};

//use super::{Battery, Cpus, Gpu};
use super::{OnLoad, OnPowerEvent, OnResume, OnSet, OnUnload, SettingError};
use super::{TBattery, TCpus, TGeneral, TGpu};
use crate::persist::{FileJson, SettingsJson};
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
    pub app_id: u64,
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

impl OnPowerEvent for General {}

impl OnLoad for General {
    fn on_load(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl OnUnload for General {
    fn on_unload(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

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

    fn app_id(&mut self) -> &'_ mut u64 {
        &mut self.app_id
    }

    fn get_app_id(&self) -> u64 {
        self.app_id
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
        let json_path = crate::utility::settings_dir().join(self.get_path());
        if let Ok(file) = crate::persist::FileJson::open(json_path) {
            file.variants
                .into_iter()
                .map(|(id, conf)| crate::api::VariantInfo {
                    id: id.to_string(),
                    name: conf.name,
                    id_num: id,
                })
                .collect()
        } else {
            vec![self.get_variant_info()]
        }
    }

    fn add_variant(
        &self,
        variant: crate::persist::SettingsJson,
    ) -> Result<Vec<crate::api::VariantInfo>, SettingError> {
        let variant_name = variant.name.clone();
        let json_path = crate::utility::settings_dir().join(self.get_path());
        crate::persist::FileJson::update_variant_or_create(
            json_path,
            self.get_app_id(),
            variant,
            variant_name,
        )
        .map_err(|e| SettingError {
            msg: format!("failed to add variant: {}", e),
            setting: SettingVariant::General,
        })
        .map(|file| {
            file.0
                .variants
                .into_iter()
                .map(|(id, conf)| crate::api::VariantInfo {
                    id: id.to_string(),
                    name: conf.name,
                    id_num: id,
                })
                .collect()
        })
    }

    fn get_variant_info(&self) -> crate::api::VariantInfo {
        log::debug!(
            "Current variant `{}` ({})",
            self.variant_name,
            self.variant_id
        );
        crate::api::VariantInfo {
            id: self.variant_id.to_string(),
            name: self.variant_name.clone(),
            id_num: self.variant_id,
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
    pub fn from_json(name: String, other: SettingsJson, json_path: PathBuf, app_id: u64) -> Self {
        let x = super::Driver::init(name, &other, json_path.clone(), app_id);
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

    pub fn system_default(
        json_path: PathBuf,
        app_id: u64,
        name: String,
        variant_id: u64,
        variant_name: String,
    ) -> Self {
        let driver =
            super::Driver::system_default(json_path, app_id, name, variant_id, variant_name);
        Self {
            general: driver.general,
            cpus: driver.cpus,
            gpu: driver.gpu,
            battery: driver.battery,
        }
    }

    pub fn load_system_default(&mut self, name: String, variant_id: u64, variant_name: String) {
        let driver = super::Driver::system_default(
            self.general.get_path().to_owned(),
            self.general.get_app_id(),
            name,
            variant_id,
            variant_name,
        );
        self.cpus = driver.cpus;
        self.gpu = driver.gpu;
        self.battery = driver.battery;
        self.general = driver.general;
    }

    pub fn get_variant<'a>(
        settings_file: &'a FileJson,
        variant_id: u64,
        variant_name: String,
    ) -> Result<&'a SettingsJson, SettingError> {
        if let Some(variant) = settings_file.variants.get(&variant_id) {
            Ok(variant)
        } else if variant_id == 0 {
            // special case: requesting primary variant for settings with non-persistent primary
            let mut valid_ids: Vec<&u64> = settings_file.variants.keys().collect();
            valid_ids.sort();
            if let Some(id) = valid_ids.get(0) {
                Ok(settings_file
                    .variants
                    .get(id)
                    .expect("variant id key magically disappeared"))
            } else {
                Err(SettingError {
                    msg: format!(
                        "Cannot get variant `{}` (id:{}) from empty settings file",
                        variant_name, variant_id
                    ),
                    setting: SettingVariant::General,
                })
            }
        } else {
            Err(SettingError {
                msg: format!(
                    "Cannot get non-existent variant `{}` (id:{})",
                    variant_name, variant_id
                ),
                setting: SettingVariant::General,
            })
        }
    }

    pub fn load_file(
        &mut self,
        filename: PathBuf,
        app_id: u64,
        name: String,
        variant: u64,
        variant_name: String,
        system_defaults: bool,
    ) -> Result<bool, SettingError> {
        let json_path = crate::utility::settings_dir().join(&filename);
        if json_path.exists() {
            if variant == u64::MAX {
                log::debug!(
                    "Creating new variant `{}` in existing settings file {}",
                    variant_name,
                    json_path.display()
                );
                self.create_and_load_variant(&json_path, app_id, variant_name)?;
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
                    self.general.variant_name(settings_json.name.clone());
                    self.general.variant_id(settings_json.variant);
                } else {
                    let x = super::Driver::init(name, settings_json, json_path.clone(), app_id);
                    log::info!(
                        "Loaded settings with drivers general:{:?},cpus:{:?},gpu:{:?},battery:{:?}",
                        x.general.provider(),
                        x.cpus.provider(),
                        x.gpu.provider(),
                        x.battery.provider()
                    );
                    self.general = x.general;
                    self.cpus = x.cpus;
                    self.gpu = x.gpu;
                    self.battery = x.battery;
                }
            }
        } else {
            if system_defaults {
                self.load_system_default(name, variant, variant_name.clone());
            } else {
                self.general.name(name);
                self.general.variant_name(variant_name.clone());
                self.general.variant_id(variant);
            }
            *self.general.persistent() = false;
            if variant == u64::MAX {
                log::debug!(
                    "Creating new variant `{}` in new settings file {}",
                    variant_name,
                    json_path.display()
                );
                self.create_and_load_variant(&json_path, app_id, variant_name)?;
            }
        }
        *self.general.app_id() = app_id;
        self.general.path(filename);
        Ok(*self.general.persistent())
    }

    fn create_and_load_variant(
        &mut self,
        json_path: &PathBuf,
        app_id: u64,
        variant_name: String,
    ) -> Result<(), SettingError> {
        *self.general.persistent() = true;
        self.general.variant_id(u64::MAX);
        self.general.variant_name(variant_name.clone());
        let (_file_json, new_variant) = FileJson::update_variant_or_create(
            json_path,
            app_id,
            self.json(),
            self.general.get_name().to_owned(),
        )
        .map_err(|e| SettingError {
            msg: format!("Failed to open settings {}: {}", json_path.display(), e),
            setting: SettingVariant::General,
        })?;
        self.general.variant_id(new_variant.variant);
        self.general.variant_name(new_variant.name);
        Ok(())
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

impl OnPowerEvent for Settings {
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

impl OnLoad for Settings {
    fn on_load(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        self.general
            .on_load()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.battery
            .on_load()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.cpus
            .on_load()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.gpu
            .on_load()
            .unwrap_or_else(|mut e| errors.append(&mut e));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl OnUnload for Settings {
    fn on_unload(&mut self) -> Result<(), Vec<SettingError>> {
        let mut errors = Vec::new();

        self.general
            .on_unload()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.battery
            .on_unload()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.cpus
            .on_unload()
            .unwrap_or_else(|mut e| errors.append(&mut e));
        self.gpu
            .on_unload()
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
