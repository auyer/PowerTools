use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use usdpl_back::core::serdes::Primitive;
use usdpl_back::AsyncCallable;

use super::handler::{ApiMessage, GeneralMessage};

const BASE_URL: &'static str = "http://powertools.ngni.us";

/// Get search results web method
pub fn search_by_app_id() -> impl AsyncCallable {
    let getter = move || {
        move |steam_app_id: u32| {
            let req_url = format!("{}/api/setting/by_app_id/{}", BASE_URL, steam_app_id);
            match ureq::get(&req_url).call() {
                Ok(response) => {
                    let json_res: std::io::Result<Vec<community_settings_core::v1::Metadata>> = response.into_json();
                    match json_res {
                        Ok(search_results) => {
                            // search results may be quite large, so let's do the JSON string conversion in the background (blocking) thread
                            match serde_json::to_string(&search_results) {
                                Err(e) => log::error!("Cannot convert search results from `{}` to JSON: {}", req_url, e),
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

fn web_config_to_settings_json(meta: community_settings_core::v1::Metadata) -> crate::persist::SettingsJson {
    crate::persist::SettingsJson {
        version: crate::persist::LATEST_VERSION,
        name: meta.name,
        variant: u64::MAX, // TODO maybe change this to use the 64 low bits of id (u64::MAX will cause it to generate a new id when added to file variant map
        persistent: true,
        cpus: meta.config.cpus.into_iter().map(|cpu| crate::persist::CpuJson {
            online: cpu.online,
            clock_limits: cpu.clock_limits.map(|lim| crate::persist::MinMaxJson {
                min: lim.min,
                max: lim.max,
            }),
            governor: cpu.governor,
            root: None,
        }).collect(),
        gpu: crate::persist::GpuJson {
            fast_ppt: meta.config.gpu.fast_ppt,
            slow_ppt: meta.config.gpu.slow_ppt,
            tdp: meta.config.gpu.tdp,
            tdp_boost: meta.config.gpu.tdp_boost,
            clock_limits: meta.config.gpu.clock_limits.map(|lim| crate::persist::MinMaxJson {
                min: lim.min,
                max: lim.max,
            }),
            slow_memory: meta.config.gpu.slow_memory,
            root: None,
        },
        battery: crate::persist::BatteryJson {
            charge_rate: meta.config.battery.charge_rate,
            charge_mode: meta.config.battery.charge_mode,
            events: meta.config.battery.events.into_iter().map(|be| crate::persist::BatteryEventJson {
                charge_rate: be.charge_rate,
                charge_mode: be.charge_mode,
                trigger: be.trigger,
            }).collect(),
            root: None,
        },
        provider: Some(crate::persist::DriverJson::AutoDetect),
    }
}

/// Download config web method
pub fn download_new_config(sender: Sender<ApiMessage>) -> impl AsyncCallable {
    let sender = Arc::new(Mutex::new(sender)); // Sender is not Sync; this is required for safety
    let getter = move || {
        let sender2 = sender.clone();
        move |id: u128| {
            let req_url = format!("{}/api/setting/by_id/{}", BASE_URL, id);
            match ureq::get(&req_url).call() {
                Ok(response) => {
                    let json_res: std::io::Result<community_settings_core::v1::Metadata> = response.into_json();
                    match json_res {
                        Ok(meta) => {
                            let (tx, rx) = mpsc::channel();
                            let callback =
                                move |values: Vec<super::VariantInfo>| tx.send(values).expect("download_new_config callback send failed");
                            sender2
                                .lock()
                                .unwrap()
                                .send(ApiMessage::General(GeneralMessage::AddVariant(web_config_to_settings_json(meta), Box::new(callback))))
                                .expect("download_new_config send failed");
                            return rx.recv().expect("download_new_config callback recv failed");
                        }
                        Err(e) => {
                            log::error!("Cannot parse response from `{}`: {}", req_url, e)
                        }
                    }
                }
                Err(e) => log::warn!("Cannot get setting result from `{}`: {}", req_url, e),
            }
            vec![]
        }
    };
    super::async_utils::AsyncIsh {
        trans_setter: |params| {
            if let Some(Primitive::String(id)) = params.get(0) {
                match id.parse::<u128>() {
                    Ok(id) => Ok(id),
                    Err(e) => Err(format!("download_new_config non-u128 string parameter 0: {} (got `{}`)", e, id))
                }
            } else {
                Err("download_new_config missing/invalid parameter 0".to_owned())
            }
        },
        set_get: getter,
        trans_getter: |result| {
            let mut output = Vec::with_capacity(result.len());
            for status in result.iter() {
                output.push(Primitive::Json(serde_json::to_string(status).expect("Failed to serialize variant info to JSON")));
            }
            output
        },
    }
}
