//use std::sync::{LockResult, MutexGuard};
//use std::fs::{Permissions, metadata};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;

use serde::{Deserialize, Serialize};
use chrono::{offset::Utc, DateTime};

/*pub fn unwrap_lock<'a, T: Sized>(
    result: LockResult<MutexGuard<'a, T>>,
    lock_name: &str,
) -> MutexGuard<'a, T> {
    match result {
        Ok(x) => x,
        Err(e) => {
            log::error!("Failed to acquire {} lock: {}", lock_name, e);
            panic!("Failed to acquire {} lock: {}", lock_name, e);
        }
    }
}*/

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedData<T> {
    pub data: T,
    pub updated: DateTime<Utc>,
}

impl <T> CachedData<T> {
    pub fn needs_update(&self, max_age: std::time::Duration) -> bool {
        self.updated < (Utc::now() - max_age)
    }
}

pub fn ron_pretty_config() -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::default()
        .struct_names(true)
        .compact_arrays(true)
}

#[allow(dead_code)]
pub fn settings_dir_old() -> std::path::PathBuf {
    usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/tmp/".into())
        .join(".config/powertools/")
}

pub fn settings_dir() -> std::path::PathBuf {
    usdpl_back::api::decky::settings_dir()
        .unwrap_or_else(|_| "/tmp/".to_owned())
        .into()
}

pub fn chown_settings_dir() -> std::io::Result<()> {
    let dir = settings_dir();
    #[cfg(feature = "decky")]
    let deck_user = usdpl_back::api::decky::user().map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Decky missing deck user's username",
        )
    })?;
    #[cfg(not(feature = "decky"))]
    let deck_user = "deck".to_owned();
    // FIXME this shouldn't need to invoke a command
    let output = std::process::Command::new("id")
        .args(["-u", &deck_user])
        .output()?;
    let uid: u32 = String::from_utf8_lossy(&output.stdout)
        .parse()
        .unwrap_or(1000);
    log::info!(
        "chmod/chown ~/.config/powertools for user `{}` ({})",
        deck_user,
        uid
    );
    let permissions = PermissionsExt::from_mode(0o755);
    std::fs::set_permissions(&dir, permissions)?;
    // FIXME once merged into stable https://github.com/rust-lang/rust/issues/88989
    //std::os::unix::fs::chown(&dir, Some(uid), Some(uid))
    std::process::Command::new("chown")
        .args([
            "-R",
            &format!("{}:{}", deck_user, deck_user),
            &dir.to_str().unwrap_or("."),
        ])
        .output()?;
    Ok(())
}

fn version_filepath() -> std::path::PathBuf {
    settings_dir().join(".version")
}

pub fn save_version_file() -> std::io::Result<usize> {
    let path = version_filepath();
    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }
    std::fs::File::create(path)?.write(crate::consts::PACKAGE_VERSION.as_bytes())
}

pub fn read_version_file() -> String {
    let path = version_filepath();
    match std::fs::File::open(path) {
        Ok(mut file) => {
            let mut read_version = String::new();
            match file.read_to_string(&mut read_version) {
                Ok(_) => read_version,
                Err(e) => {
                    log::warn!("Cannot read version file str: {}", e);
                    crate::consts::PACKAGE_VERSION.to_owned()
                }
            }
        }
        Err(e) => {
            log::warn!("Cannot read version file: {}", e);
            crate::consts::PACKAGE_VERSION.to_owned()
        }
    }
}

pub fn ioperm_power_ec() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        let temp_ec = smokepatio::ec::unnamed_power::UnnamedPowerEC::new();
        libc::ioperm(temp_ec.ec().data() as _, 1, 1);
        libc::ioperm(temp_ec.ec().cmd() as _, 1, 1);
    }
}

#[cfg(test)]
mod generate {
    #[test]
    fn generate_default_limits_override() {
        let limits = limits_core::json_v2::Limits {
            cpu: limits_core::json_v2::Limit {
                provider: limits_core::json_v2::CpuLimitType::SteamDeck,
                limits: limits_core::json_v2::GenericCpusLimit::default_for(
                    limits_core::json_v2::CpuLimitType::SteamDeck,
                ),
            },
            gpu: limits_core::json_v2::Limit {
                provider: limits_core::json_v2::GpuLimitType::SteamDeck,
                limits: limits_core::json_v2::GenericGpuLimit::default_for(
                    limits_core::json_v2::GpuLimitType::SteamDeck,
                ),
            },
            battery: limits_core::json_v2::Limit {
                provider: limits_core::json_v2::BatteryLimitType::SteamDeck,
                limits: limits_core::json_v2::GenericBatteryLimit::default_for(
                    limits_core::json_v2::BatteryLimitType::SteamDeck,
                ),
            },
        };
        let output_file =
            std::fs::File::create(format!("../{}", crate::consts::LIMITS_OVERRIDE_FILE)).unwrap();
        ron::ser::to_writer_pretty(output_file, &limits, crate::utility::ron_pretty_config())
            .unwrap();
    }

    #[test]
    fn generate_default_minimal_save_file() {
        let mut mini_variants = std::collections::HashMap::with_capacity(2);
        mini_variants.insert(
            0,
            crate::persist::SettingsJson {
                version: 0,
                name: crate::consts::DEFAULT_SETTINGS_VARIANT_NAME.to_owned(),
                variant: 0,
                persistent: false,
                cpus: vec![crate::persist::CpuJson::default(); 8],
                gpu: crate::persist::GpuJson::default(),
                battery: crate::persist::BatteryJson::default(),
                provider: None,
            },
        );
        mini_variants.insert(
            42,
            crate::persist::SettingsJson {
                version: 0,
                name: "FortySecondary".to_owned(),
                variant: 42,
                persistent: false,
                cpus: vec![crate::persist::CpuJson::default(); 8],
                gpu: crate::persist::GpuJson::default(),
                battery: crate::persist::BatteryJson::default(),
                provider: None,
            },
        );
        let savefile = crate::persist::FileJson {
            version: 0,
            app_id: 0,
            name: crate::consts::DEFAULT_SETTINGS_NAME.to_owned(),
            variants: mini_variants,
        };
        let output_file =
            std::fs::File::create(format!("../{}", crate::consts::DEFAULT_SETTINGS_FILE)).unwrap();
        ron::ser::to_writer_pretty(output_file, &savefile, crate::utility::ron_pretty_config())
            .unwrap();
    }
}
