use std::fs::File;

use regex::RegexBuilder;

use limits_core::json_v2::{BatteryLimitType, CpuLimitType, GpuLimitType, Limits};

use crate::persist::{DriverJson, SettingsJson};
use crate::settings::{Driver, General, ProviderBuilder, TBattery, TCpus, TGeneral, TGpu};

fn get_limits() -> limits_core::json_v2::Base {
    super::limits_worker::get_limits_cached()
}

fn get_limits_overrides() -> Option<Limits> {
    let limits_override_path = super::utility::limits_override_path();
    match File::open(&limits_override_path) {
        Ok(f) => match ron::de::from_reader(f) {
            Ok(lim) => Some(lim),
            Err(e) => {
                log::warn!(
                    "Failed to parse limits override file `{}`, cannot use for auto_detect: {}",
                    limits_override_path.display(),
                    e
                );
                None
            }
        },
        Err(e) => {
            log::info!(
                "Failed to open limits override file `{}`: {}",
                limits_override_path.display(),
                e
            );
            None
        }
    }
}

#[inline]
pub fn auto_detect_provider() -> DriverJson {
    let provider = auto_detect0(
        None,
        crate::utility::settings_dir().join("autodetect.json"),
        0,
        "".to_owned(),
        0,
        crate::consts::DEFAULT_SETTINGS_VARIANT_NAME.to_owned(),
    )
    .battery
    .provider();
    //log::info!("Detected device automatically, compatible driver: {:?}", provider);
    provider
}

/// Device detection logic
pub fn auto_detect0(
    settings_opt: Option<&SettingsJson>,
    json_path: std::path::PathBuf,
    app_id: u64,
    name: String,
    variant_id: u64,
    variant_name: String,
) -> Driver {
    let mut general_driver = Box::new(General {
        persistent: false,
        path: json_path,
        app_id,
        name,
        variant_id,
        variant_name,
        driver: DriverJson::AutoDetect,
    });

    let cpu_info: String = usdpl_back::api::files::read_single("/proc/cpuinfo").unwrap_or_default();
    log::debug!("Read from /proc/cpuinfo:\n{}", cpu_info);
    let os_info: String =
        usdpl_back::api::files::read_single("/etc/os-release").unwrap_or_default();
    log::debug!("Read from /etc/os-release:\n{}", os_info);
    let dmi_info: String = std::process::Command::new("dmidecode")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .unwrap_or_default();
    log::debug!("Read dmidecode:\n{}", dmi_info);

    let limits = get_limits();
    let limits_override = get_limits_overrides();

    // build driver based on limits conditions
    for conf in limits.configs {
        let conditions = conf.conditions;
        let mut matches = true;
        if let Some(dmi) = &conditions.dmi {
            let pattern = RegexBuilder::new(dmi)
                .multi_line(true)
                .build()
                .expect("Invalid DMI regex");
            matches &= pattern.is_match(&dmi_info);
        }
        if let Some(cpuinfo) = &conditions.cpuinfo {
            let pattern = RegexBuilder::new(cpuinfo)
                .multi_line(true)
                .build()
                .expect("Invalid CPU regex");
            matches &= pattern.is_match(&cpu_info);
        }
        if let Some(os) = &conditions.os {
            let pattern = RegexBuilder::new(os)
                .multi_line(true)
                .build()
                .expect("Invalid OS regex");
            matches &= pattern.is_match(&os_info);
        }
        if let Some(cmd) = &conditions.command {
            match std::process::Command::new("bash")
                .args(["-c", cmd])
                .status()
            {
                Ok(status) => matches &= status.code().map(|c| c == 0).unwrap_or(false),
                Err(e) => log::warn!("Ignoring bash limits error: {}", e),
            }
        }
        if let Some(file_exists) = &conditions.file_exists {
            let exists = std::path::Path::new(file_exists).exists();
            matches &= exists;
        }

        if matches {
            let mut relevant_limits = conf.limits.clone();
            relevant_limits.apply_override(limits_override);
            if let Some(settings) = &settings_opt {
                *general_driver.persistent() = true;
                let cpu_driver: Box<dyn TCpus> = match relevant_limits.cpu.provider {
                    CpuLimitType::SteamDeck => Box::new(
                        crate::settings::steam_deck::Cpus::from_json_and_limits(
                            settings.cpus.clone(),
                            settings.version,
                            relevant_limits.cpu.limits,
                        )
                        .variant(super::super::steam_deck::Model::LCD),
                    ),
                    CpuLimitType::SteamDeckOLED => Box::new(
                        crate::settings::steam_deck::Cpus::from_json_and_limits(
                            settings.cpus.clone(),
                            settings.version,
                            relevant_limits.cpu.limits,
                        )
                        .variant(super::super::steam_deck::Model::OLED),
                    ),
                    CpuLimitType::Generic => Box::new(crate::settings::generic::Cpus::<
                        crate::settings::generic::Cpu,
                    >::from_json_and_limits(
                        settings.cpus.clone(),
                        settings.version,
                        relevant_limits.cpu.limits,
                    )),
                    CpuLimitType::GenericAMD => {
                        Box::new(crate::settings::generic_amd::Cpus::from_json_and_limits(
                            settings.cpus.clone(),
                            settings.version,
                            relevant_limits.cpu.limits,
                        ))
                    }
                    CpuLimitType::Unknown => {
                        Box::new(crate::settings::unknown::Cpus::from_json_and_limits(
                            settings.cpus.clone(),
                            settings.version,
                            relevant_limits.cpu.limits,
                        ))
                    }
                    CpuLimitType::DevMode => {
                        Box::new(crate::settings::dev_mode::Cpus::from_json_and_limits(
                            settings.cpus.clone(),
                            settings.version,
                            relevant_limits.cpu.limits,
                        ))
                    }
                };

                let gpu_driver: Box<dyn TGpu> = match relevant_limits.gpu.provider {
                    GpuLimitType::SteamDeck => Box::new(
                        crate::settings::steam_deck::Gpu::from_json_and_limits(
                            settings.gpu.clone(),
                            settings.version,
                            relevant_limits.gpu.limits,
                        )
                        .variant(super::super::steam_deck::Model::LCD),
                    ),
                    GpuLimitType::SteamDeckOLED => Box::new(
                        crate::settings::steam_deck::Gpu::from_json_and_limits(
                            settings.gpu.clone(),
                            settings.version,
                            relevant_limits.gpu.limits,
                        )
                        .variant(super::super::steam_deck::Model::OLED),
                    ),
                    GpuLimitType::Generic => {
                        Box::new(crate::settings::generic::Gpu::from_json_and_limits(
                            settings.gpu.clone(),
                            settings.version,
                            relevant_limits.gpu.limits,
                        ))
                    }
                    GpuLimitType::GenericAMD => {
                        Box::new(crate::settings::generic_amd::Gpu::from_json_and_limits(
                            settings.gpu.clone(),
                            settings.version,
                            relevant_limits.gpu.limits,
                        ))
                    }
                    GpuLimitType::Unknown => {
                        Box::new(crate::settings::unknown::Gpu::from_json_and_limits(
                            settings.gpu.clone(),
                            settings.version,
                            relevant_limits.gpu.limits,
                        ))
                    }
                    GpuLimitType::DevMode => {
                        Box::new(crate::settings::dev_mode::Gpu::from_json_and_limits(
                            settings.gpu.clone(),
                            settings.version,
                            relevant_limits.gpu.limits,
                        ))
                    }
                };
                let battery_driver: Box<dyn TBattery> = match relevant_limits.battery.provider {
                    BatteryLimitType::SteamDeck => Box::new(
                        crate::settings::steam_deck::Battery::from_json_and_limits(
                            settings.battery.clone(),
                            settings.version,
                            relevant_limits.battery.limits,
                        )
                        .variant(super::super::steam_deck::Model::LCD),
                    ),
                    BatteryLimitType::SteamDeckOLED => Box::new(
                        crate::settings::steam_deck::Battery::from_json_and_limits(
                            settings.battery.clone(),
                            settings.version,
                            relevant_limits.battery.limits,
                        )
                        .variant(super::super::steam_deck::Model::OLED),
                    ),
                    BatteryLimitType::Generic => {
                        Box::new(crate::settings::generic::Battery::from_json_and_limits(
                            settings.battery.clone(),
                            settings.version,
                            relevant_limits.battery.limits,
                        ))
                    }
                    BatteryLimitType::Unknown => {
                        Box::new(crate::settings::unknown::Battery::from_json_and_limits(
                            settings.battery.clone(),
                            settings.version,
                            relevant_limits.battery.limits,
                        ))
                    }
                    BatteryLimitType::DevMode => {
                        Box::new(crate::settings::dev_mode::Battery::from_json_and_limits(
                            settings.battery.clone(),
                            settings.version,
                            relevant_limits.battery.limits,
                        ))
                    }
                };

                return Driver {
                    general: general_driver,
                    cpus: cpu_driver,
                    gpu: gpu_driver,
                    battery: battery_driver,
                };
            } else {
                let cpu_driver: Box<dyn TCpus> = match relevant_limits.cpu.provider {
                    CpuLimitType::SteamDeck => Box::new(
                        crate::settings::steam_deck::Cpus::from_limits(relevant_limits.cpu.limits)
                            .variant(super::super::steam_deck::Model::LCD),
                    ),
                    CpuLimitType::SteamDeckOLED => Box::new(
                        crate::settings::steam_deck::Cpus::from_limits(relevant_limits.cpu.limits)
                            .variant(super::super::steam_deck::Model::OLED),
                    ),
                    CpuLimitType::Generic => Box::new(crate::settings::generic::Cpus::<
                        crate::settings::generic::Cpu,
                    >::from_limits(
                        relevant_limits.cpu.limits
                    )),
                    CpuLimitType::GenericAMD => Box::new(
                        crate::settings::generic_amd::Cpus::from_limits(relevant_limits.cpu.limits),
                    ),
                    CpuLimitType::Unknown => Box::new(crate::settings::unknown::Cpus::from_limits(
                        relevant_limits.cpu.limits,
                    )),
                    CpuLimitType::DevMode => Box::new(
                        crate::settings::dev_mode::Cpus::from_limits(relevant_limits.cpu.limits),
                    ),
                };
                let gpu_driver: Box<dyn TGpu> = match relevant_limits.gpu.provider {
                    GpuLimitType::SteamDeck => Box::new(
                        crate::settings::steam_deck::Gpu::from_limits(relevant_limits.gpu.limits)
                            .variant(super::super::steam_deck::Model::LCD),
                    ),
                    GpuLimitType::SteamDeckOLED => Box::new(
                        crate::settings::steam_deck::Gpu::from_limits(relevant_limits.gpu.limits)
                            .variant(super::super::steam_deck::Model::OLED),
                    ),
                    GpuLimitType::Generic => Box::new(crate::settings::generic::Gpu::from_limits(
                        relevant_limits.gpu.limits,
                    )),
                    GpuLimitType::GenericAMD => Box::new(
                        crate::settings::generic_amd::Gpu::from_limits(relevant_limits.gpu.limits),
                    ),
                    GpuLimitType::Unknown => Box::new(crate::settings::unknown::Gpu::from_limits(
                        relevant_limits.gpu.limits,
                    )),
                    GpuLimitType::DevMode => Box::new(crate::settings::dev_mode::Gpu::from_limits(
                        relevant_limits.gpu.limits,
                    )),
                };
                let battery_driver: Box<dyn TBattery> = match relevant_limits.battery.provider {
                    BatteryLimitType::SteamDeck => Box::new(
                        crate::settings::steam_deck::Battery::from_limits(
                            relevant_limits.battery.limits,
                        )
                        .variant(super::super::steam_deck::Model::LCD),
                    ),
                    BatteryLimitType::SteamDeckOLED => Box::new(
                        crate::settings::steam_deck::Battery::from_limits(
                            relevant_limits.battery.limits,
                        )
                        .variant(super::super::steam_deck::Model::OLED),
                    ),
                    BatteryLimitType::Generic => {
                        Box::new(crate::settings::generic::Battery::from_limits(
                            relevant_limits.battery.limits,
                        ))
                    }
                    BatteryLimitType::Unknown => {
                        Box::new(crate::settings::unknown::Battery::from_limits(
                            relevant_limits.battery.limits,
                        ))
                    }
                    BatteryLimitType::DevMode => {
                        Box::new(crate::settings::dev_mode::Battery::from_limits(
                            relevant_limits.battery.limits,
                        ))
                    }
                };
                return Driver {
                    general: general_driver,
                    cpus: cpu_driver,
                    gpu: gpu_driver,
                    battery: battery_driver,
                };
            }
        }
    }

    Driver {
        general: general_driver,
        cpus: Box::new(crate::settings::unknown::Cpus::system_default()),
        gpu: Box::new(crate::settings::unknown::Gpu::system_default()),
        battery: Box::new(crate::settings::unknown::Battery),
    }
}
