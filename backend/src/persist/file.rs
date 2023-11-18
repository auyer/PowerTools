use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::SerdeError;
use super::SettingsJson;

#[derive(Serialize, Deserialize)]
pub struct FileJson {
    pub version: u64,
    pub name: String,
    pub variants: HashMap<String, SettingsJson>,
}

impl FileJson {
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), SerdeError> {
        let path = path.as_ref();

        if !self.variants.is_empty() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(SerdeError::Io)?;
            }
            let mut file = std::fs::File::create(path).map_err(SerdeError::Io)?;
            ron::ser::to_writer_pretty(&mut file, &self, crate::utility::ron_pretty_config()).map_err(|e| SerdeError::Serde(e.into()))
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

    pub fn update_variant_or_create<P: AsRef<std::path::Path>>(path: P, setting: SettingsJson, given_name: String) -> Result<(), SerdeError> {
        if !setting.persistent {
            return Ok(())
        }
        let path = path.as_ref();

        let file = if path.exists() {
            let mut file = Self::open(path)?;
            file.variants.insert(setting.variant.to_string(), setting);
            file
        } else {
            let mut setting_variants = HashMap::with_capacity(1);
            setting_variants.insert(setting.variant.to_string(), setting);
            Self {
                version: 0,
                name: given_name,
                variants: setting_variants,
            }
        };

        file.save(path)
    }
}
