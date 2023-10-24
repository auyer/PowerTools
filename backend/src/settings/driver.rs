use super::{auto_detect0, TBattery, TCpus, TGeneral, TGpu};
use crate::persist::{DriverJson, SettingsJson};

pub struct Driver {
    pub general: Box<dyn TGeneral>,
    pub cpus: Box<dyn TCpus>,
    pub gpu: Box<dyn TGpu>,
    pub battery: Box<dyn TBattery>,
}

impl Driver {
    pub fn init(
        settings: SettingsJson,
        json_path: std::path::PathBuf,
    ) -> Self {
        let name_bup = settings.name.clone();
        auto_detect0(Some(settings), json_path, name_bup)
    }

    pub fn system_default(json_path: std::path::PathBuf, name: String) -> Self {
        auto_detect0(None, json_path, name)
    }
}

// sshhhh, this function isn't here ;)
#[inline]
pub fn maybe_do_button() {
    match super::auto_detect_provider() {
        DriverJson::SteamDeck | DriverJson::SteamDeckAdvance => {
            crate::settings::steam_deck::flash_led();
        }
        DriverJson::Generic | DriverJson::GenericAMD => {
            log::warn!("You need to come up with something fun on generic")
        }
        DriverJson::Unknown => log::warn!("Can't do button activities on unknown platform"),
        DriverJson::AutoDetect => log::warn!("WTF, why is auto_detect detecting AutoDetect???"),
    }
}
