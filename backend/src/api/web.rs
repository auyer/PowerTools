use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, RwLock};
use usdpl_back::core::serdes::Primitive;
use usdpl_back::AsyncCallable;

use super::handler::{ApiMessage, GeneralMessage};

const BASE_URL_FALLBACK: &'static str = "https://powertools.ngni.us";
static BASE_URL: RwLock<Option<String>> = RwLock::new(None);

pub fn set_base_url(base_url: String) {
    *BASE_URL.write().expect("Failed to acquire write lock for store base url") = Some(base_url);
}

fn get_base_url() -> String {
    BASE_URL.read().expect("Failed to acquire read lock for store base url")
        .clone()
        .unwrap_or_else(|| BASE_URL_FALLBACK.to_owned())
}

fn url_search_by_app_id(steam_app_id: u32) -> String {
    format!("{}/api/setting/by_app_id/{}", get_base_url(), steam_app_id)
}

fn url_download_config_by_id(id: u128) -> String {
    format!("{}/api/setting/by_id/{}", get_base_url(), id)
}

fn url_upload_config() -> String {
    format!("{}/api/setting", get_base_url())
}

/// Get search results web method
pub fn search_by_app_id() -> impl AsyncCallable {
    let getter = move || {
        move |steam_app_id: u32| {
            let req_url = url_search_by_app_id(steam_app_id);
            match ureq::get(&req_url).call() {
                Ok(response) => {
                    let json_res: std::io::Result<Vec<community_settings_core::v1::Metadata>> =
                        response.into_json();
                    match json_res {
                        Ok(search_results) => {
                            // search results may be quite large, so let's do the JSON string conversion in the background (blocking) thread
                            match serde_json::to_string(&search_results) {
                                Err(e) => log::error!(
                                    "Cannot convert search results from `{}` to JSON: {}",
                                    req_url,
                                    e
                                ),
                                Ok(s) => return s,
                            }
                        }
                        Err(e) => {
                            log::error!("Cannot parse response from `{}`: {}", req_url, e)
                        }
                    }
                }
                Err(e) => log::warn!("Cannot get search results from `{}`: {}", req_url, e),
            }
            "[]".to_owned()
        }
    };
    super::async_utils::AsyncIsh {
        trans_setter: |params| {
            if let Some(Primitive::F64(app_id)) = params.get(0) {
                Ok(*app_id as u32)
            } else {
                Err("search_by_app_id missing/invalid parameter 0".to_owned())
            }
        },
        set_get: getter,
        trans_getter: |result| vec![Primitive::Json(result)],
    }
}

fn web_config_to_settings_json(
    meta: community_settings_core::v1::Metadata,
) -> crate::persist::SettingsJson {
    crate::persist::SettingsJson {
        version: crate::persist::LATEST_VERSION,
        name: meta.name,
        variant: u64::MAX, // TODO maybe change this to use the 64 low bits of id (u64::MAX will cause it to generate a new id when added to file variant map
        persistent: true,
        cpus: meta
            .config
            .cpus
            .into_iter()
            .map(|cpu| crate::persist::CpuJson {
                online: cpu.online,
                clock_limits: cpu.clock_limits.map(|lim| crate::persist::MinMaxJson {
                    min: lim.min,
                    max: lim.max,
                }),
                governor: cpu.governor,
                root: None,
            })
            .collect(),
        gpu: crate::persist::GpuJson {
            fast_ppt: meta.config.gpu.fast_ppt,
            slow_ppt: meta.config.gpu.slow_ppt,
            tdp: meta.config.gpu.tdp,
            tdp_boost: meta.config.gpu.tdp_boost,
            clock_limits: meta
                .config
                .gpu
                .clock_limits
                .map(|lim| crate::persist::MinMaxJson {
                    min: lim.min,
                    max: lim.max,
                }),
            memory_clock: meta.config.gpu.memory_clock,
            root: None,
        },
        battery: crate::persist::BatteryJson {
            charge_rate: meta.config.battery.charge_rate,
            charge_mode: meta.config.battery.charge_mode,
            events: meta
                .config
                .battery
                .events
                .into_iter()
                .map(|be| crate::persist::BatteryEventJson {
                    charge_rate: be.charge_rate,
                    charge_mode: be.charge_mode,
                    trigger: be.trigger,
                })
                .collect(),
            root: None,
        },
        provider: Some(crate::persist::DriverJson::AutoDetect),
    }
}

fn download_config(id: u128) -> std::io::Result<community_settings_core::v1::Metadata> {
    let req_url = url_download_config_by_id(id);
    let response = ureq::get(&req_url).call().map_err(|e| {
        log::warn!("GET to {} failed: {}", req_url, e);
        std::io::Error::new(std::io::ErrorKind::ConnectionAborted, e)
    })?;
    response.into_json()
}

pub fn upload_settings(
    id: u64,
    user_id: String,
    username: String,
    settings: crate::persist::SettingsJson,
) {
    log::info!(
        "Uploading settings {} by {} ({})",
        settings.name,
        username,
        user_id
    );
    let user_id: u64 = match user_id.parse() {
        Ok(id) => id,
        Err(e) => {
            log::error!(
                "Failed to parse `{}` as u64: {} (aborted upload_settings very early)",
                user_id,
                e
            );
            return;
        }
    };
    let meta = settings_to_web_config(id as _, user_id, username, settings);
    if let Err(e) = upload_config(meta) {
        log::error!("Failed to upload settings: {}", e);
    }
}

fn settings_to_web_config(
    app_id: u32,
    user_id: u64,
    username: String,
    settings: crate::persist::SettingsJson,
) -> community_settings_core::v1::Metadata {
    community_settings_core::v1::Metadata {
        name: settings.name,
        steam_app_id: app_id,
        steam_user_id: user_id,
        steam_username: username,
        tags: vec!["wip".to_owned()],
        id: "".to_owned(),
        config: community_settings_core::v1::Config {
            cpus: settings
                .cpus
                .into_iter()
                .map(|cpu| community_settings_core::v1::Cpu {
                    online: cpu.online,
                    clock_limits: cpu
                        .clock_limits
                        .map(|lim| community_settings_core::v1::MinMax {
                            min: lim.min,
                            max: lim.max,
                        }),
                    governor: cpu.governor,
                })
                .collect(),
            gpu: community_settings_core::v1::Gpu {
                fast_ppt: settings.gpu.fast_ppt,
                slow_ppt: settings.gpu.slow_ppt,
                tdp: settings.gpu.tdp,
                tdp_boost: settings.gpu.tdp_boost,
                clock_limits: settings.gpu.clock_limits.map(|lim| {
                    community_settings_core::v1::MinMax {
                        min: lim.min,
                        max: lim.max,
                    }
                }),
                memory_clock: settings.gpu.memory_clock,
            },
            battery: community_settings_core::v1::Battery {
                charge_rate: settings.battery.charge_rate,
                charge_mode: settings.battery.charge_mode,
                events: settings
                    .battery
                    .events
                    .into_iter()
                    .map(|batt_ev| community_settings_core::v1::BatteryEvent {
                        trigger: batt_ev.trigger,
                        charge_rate: batt_ev.charge_rate,
                        charge_mode: batt_ev.charge_mode,
                    })
                    .collect(),
            },
        },
    }
}

fn upload_config(config: community_settings_core::v1::Metadata) -> std::io::Result<()> {
    let req_url = url_upload_config();
    ureq::post(&req_url)
        .send_json(&config)
        .map_err(|e| {
            log::warn!("POST to {} failed: {}", req_url, e);
            std::io::Error::new(std::io::ErrorKind::ConnectionAborted, e)
        })
        .map(|_| ())
}

/// Download config web method
pub fn download_new_config(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move |id: u128| {
            match download_config(id) {
                Ok(meta) => {
                    let (tx, rx) = mpsc::channel();
                    let callback = move |values: Vec<super::VariantInfo>| {
                        tx.send(values)
                            .expect("download_new_config callback send failed")
                    };
                    sender2
                        .lock()
                        .unwrap()
                        .send(ApiMessage::General(GeneralMessage::AddVariant(
                            web_config_to_settings_json(meta),
                            Box::new(callback),
                        )))
                        .expect("download_new_config send failed");
                    return rx.recv().expect("download_new_config callback recv failed");
                }
                Err(e) => {
                    log::error!("Invalid response from download: {}", e);
                }
            }
            vec![]
        }
    };
    super::async_utils::AsyncIsh {
        trans_setter: |params| {
            if let Some(Primitive::String(id)) = params.get(0) {
                match id.parse::<u128>() {
                    Ok(id) => Ok(id),
                    Err(e) => Err(format!(
                        "download_new_config non-u128 string parameter 0: {} (got `{}`)",
                        e, id
                    )),
                }
            } else {
                Err("download_new_config missing/invalid parameter 0".to_owned())
            }
        },
        set_get: getter,
        trans_getter: |result| {
            let mut output = Vec::with_capacity(result.len());
            for status in result.iter() {
                output.push(Primitive::Json(
                    serde_json::to_string(status)
                        .expect("Failed to serialize variant info to JSON"),
                ));
            }
            output
        },
    }
}

/// Upload currently-loaded variant
pub fn upload_current_variant(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move |(steam_id, steam_username): (String, String)| {
            sender2
                .lock()
                .unwrap()
                .send(ApiMessage::UploadCurrentVariant(steam_id, steam_username))
                .expect("upload_current_variant send failed");
            true
        }
    };
    super::async_utils::AsyncIsh {
        trans_setter: |params| {
            if let Some(Primitive::String(steam_id)) = params.get(0) {
                if let Some(Primitive::String(steam_username)) = params.get(1) {
                    Ok((steam_id.to_owned(), steam_username.to_owned()))
                } else {
                    Err("upload_current_variant missing/invalid parameter 1".to_owned())
                }
            } else {
                Err("upload_current_variant missing/invalid parameter 0".to_owned())
            }
        },
        set_get: getter,
        trans_getter: |result| vec![result.into()],
    }
}
