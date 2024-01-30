mod battery;
mod cpu;
mod gpu;
mod power_dpm_force;
mod util;

pub use battery::Battery;
pub use cpu::Cpus;
pub use gpu::Gpu;
pub(self) use power_dpm_force::{
    DPM_FORCE_LIMITS_ATTRIBUTE, POWER_DPM_FORCE_PERFORMANCE_LEVEL_MGMT,
};

#[derive(Debug, Clone, Copy)]
pub enum Model {
    LCD,
    OLED,
}

pub use util::flash_led;

fn _impl_checker() {
    fn impl_provider_builder<T: crate::settings::ProviderBuilder<J, L>, J, L>() {}

    impl_provider_builder::<
        Battery,
        crate::persist::BatteryJson,
        limits_core::json_v2::GenericBatteryLimit,
    >();
    impl_provider_builder::<
        Cpus,
        Vec<crate::persist::CpuJson>,
        limits_core::json_v2::GenericCpusLimit,
    >();
    impl_provider_builder::<Gpu, crate::persist::GpuJson, limits_core::json_v2::GenericGpuLimit>();
}
