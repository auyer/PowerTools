use std::thread::{self, JoinHandle};

use limits_core::json_v2::Base;

#[inline]
fn expired_updated_time() -> chrono::DateTime<chrono::offset::Utc> {
    chrono::offset::Utc::now() - (crate::consts::LIMITS_REFRESH_PERIOD * 2)
}

#[cfg(feature = "online")]
pub fn spawn() -> JoinHandle<()> {
    thread::spawn(move || {
        log::info!("limits_worker starting...");
        let limits_path = super::utility::limits_path();
        thread::sleep(crate::consts::LIMITS_STARTUP_WAIT);
        log::info!("limits_worker completed startup wait");
        loop {
            if (limits_path.exists() && limits_path.is_file()) || !limits_path.exists() {
                // try to load limits from file, fallback to built-in default
                let mut base = if limits_path.exists() {
                    match std::fs::File::open(&limits_path) {
                        Ok(f) => match ron::de::from_reader(f) {
                            Ok(b) => b,
                            Err(e) => {
                                log::error!("Cannot parse {}: {}", limits_path.display(), e);
                                crate::utility::CachedData {
                                    data: Base::default(),
                                    updated: expired_updated_time(),
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("Cannot open {}: {}", limits_path.display(), e);
                            crate::utility::CachedData {
                                data: Base::default(),
                                updated: expired_updated_time(),
                            }
                        }
                    }
                } else {
                    let mut base = crate::utility::CachedData {
                        data: Base::default(),
                        updated: expired_updated_time(),
                    };
                    save_base(&mut base, &limits_path);
                    base
                };
                crate::api::web::set_base_url(base.data.store.clone());
                if let Some(refresh) = &base.data.refresh {
                    if base.needs_update(crate::consts::LIMITS_REFRESH_PERIOD) {
                        // try to retrieve newer version
                        match ureq::get(refresh).call() {
                            Ok(response) => {
                                let json_res: std::io::Result<Base> = response.into_json();
                                match json_res {
                                    Ok(new_base) => {
                                        base.data = new_base;
                                        save_base(&mut base, &limits_path);
                                    }
                                    Err(e) => {
                                        log::error!("Cannot parse response from `{}`: {}", refresh, e)
                                    }
                                }
                            }
                            Err(e) => log::warn!("Cannot download limits from `{}`: {}", refresh, e),
                        }
                    }
                } else {
                    log::info!("limits_worker refresh is empty, terminating...");
                    break;
                }
            } else if !limits_path.is_file() {
                log::error!("Path for storing limits is not a file!");
            }
            thread::sleep(crate::consts::LIMITS_CHECK_PERIOD);
        }
        log::warn!("limits_worker completed!");
    })
}

#[cfg(not(feature = "online"))]
pub fn spawn() -> JoinHandle<()> {
    thread::spawn(move || {
        log::info!("limits_worker disabled...");
    })
}

pub fn get_limits_cached() ->  Base {
    let limits_path = super::utility::limits_path();
    let cached: crate::utility::CachedData<Base> = if limits_path.is_file() {
        match std::fs::File::open(&limits_path) {
            Ok(f) => match ron::de::from_reader(f) {
                Ok(b) => b,
                Err(e) => {
                    log::error!("Cannot parse {}: {}", limits_path.display(), e);
                    return Base::default();
                }
            },
            Err(e) => {
                log::error!("Cannot open {}: {}", limits_path.display(), e);
                return Base::default();
            }
        }
    } else {
        return Base::default();
    };
    cached.data
}

#[cfg(feature = "online")]
fn save_base(new_base: &mut crate::utility::CachedData<Base>, path: impl AsRef<std::path::Path>) {
    let limits_path = path.as_ref();
    new_base.updated = chrono::offset::Utc::now();
    match std::fs::File::create(&limits_path) {
        Ok(f) => {
            match ron::ser::to_writer_pretty(f, new_base, crate::utility::ron_pretty_config()) {
                Ok(_) => log::info!("Successfully saved new limits to {}", limits_path.display()),
                Err(e) => log::error!(
                    "Failed to save limits json to file `{}`: {}",
                    limits_path.display(),
                    e
                ),
            }
        }
        Err(e) => log::error!("Cannot create {}: {}", limits_path.display(), e),
    }
}
