mod detect;
pub mod driver;
mod error;
mod general;
mod min_max;
mod traits;
mod util;

pub mod generic;
pub mod generic_amd;
pub mod steam_deck;
pub mod unknown;
pub mod dev_mode;

pub use detect::{auto_detect0, auto_detect_provider, limits_worker::spawn as limits_worker_spawn, get_dev_messages};
pub use driver::Driver;
pub use general::{General, SettingVariant, Settings};
pub use min_max::{min_max_from_json, MinMax};

pub use error::SettingError;
pub use traits::{OnPowerEvent, OnResume, OnSet, PowerMode, TBattery, TCpu, TCpus, TGeneral, TGpu, ProviderBuilder};

#[cfg(test)]
mod tests {
    #[test]
    fn system_defaults_test() {
        let settings = super::Settings::system_default("idc".into(), 0, "Cool name".into(), 0, "Variant 0".into());
        println!("Loaded system settings: {:?}", settings);
    }
}
