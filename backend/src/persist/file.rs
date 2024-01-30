use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::SerdeError;
use super::SettingsJson;

#[derive(Serialize, Deserialize)]
pub struct FileJson {
    pub version: u64,
    pub name: String,
    pub app_id: u64,
    pub variants: HashMap<u64, SettingsJson>,
}

impl FileJson {
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), SerdeError> {
        let path = path.as_ref();

        if !self.variants.is_empty() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(SerdeError::Io)?;
            }
            let mut file = std::fs::File::create(path).map_err(SerdeError::Io)?;
            ron::ser::to_writer_pretty(&mut file, &self, crate::utility::ron_pretty_config())
                .map_err(|e| SerdeError::Serde(e.into()))
        } else {
            if path.exists() {
                // remove settings file when persistence is turned off, to prevent it from be loaded next time.
                std::fs::remove_file(path).map_err(SerdeError::Io)
            } else {
                Ok(())
            }
        }
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, SerdeError> {
        let mut file = std::fs::File::open(path).map_err(SerdeError::Io)?;
        ron::de::from_reader(&mut file).map_err(|e| SerdeError::Serde(e.into()))
    }

    fn next_available_id(&self) -> u64 {
        self.variants.keys().max().map(|k| k + 1).unwrap_or(0)
    }

    pub fn update_variant_or_create<P: AsRef<std::path::Path>>(
        path: P,
        app_id: u64,
        mut setting: SettingsJson,
        app_name: String,
    ) -> Result<(Self, SettingsJson), SerdeError> {
        // returns (Self, updated/created variant id)
        let path = path.as_ref();
        if !setting.persistent {
            let mut file = Self::open(path)?;

            if file.variants.contains_key(&setting.variant) {
                file.variants.remove(&setting.variant);
                file.save(path)?;
            }
            return Ok((file, setting));
        }

        let (file, variant_id) = if path.exists() {
            let mut file = Self::open(path)?;
            // Generate new (available) id if max
            if setting.variant == u64::MAX {
                setting.variant = file.next_available_id();
            }
            // Generate new name if empty
            if setting.name.is_empty() {
                setting.name = format!("Variant {}", setting.variant);
            }
            log::debug!("Inserting setting variant `{}` ({}) for app `{}` ({})", setting.name, setting.variant, file.name, app_id);
            file.variants.insert(setting.variant, setting.clone());
            (file, setting)
        } else {
            // Generate new id if max
            if setting.variant == u64::MAX {
                setting.variant = 1;
            }
            // Generate new name if empty
            if setting.name.is_empty() {
                setting.name = format!("Variant {}", setting.variant);
            }
            log::debug!("Creating new setting variant `{}` ({}) for app `{}` ({})", setting.name, setting.variant, app_name, app_id);
            let mut setting_variants = HashMap::with_capacity(1);
            setting_variants.insert(setting.variant, setting.clone());
            (Self {
                version: 0,
                app_id: app_id,
                name: app_name,
                variants: setting_variants,
            }, setting)
        };

        file.save(path)?;
        Ok((file, variant_id))
    }
}
