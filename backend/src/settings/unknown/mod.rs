mod battery;
mod cpu;
mod gpu;

pub use battery::Battery;
pub use cpu::{Cpu, Cpus};
pub use gpu::Gpu;

fn _impl_checker() {
    fn impl_provider_builder<T: crate::settings::ProviderBuilder<J, L>, J, L>() {}

    impl_provider_builder::<Battery, crate::persist::BatteryJson, limits_core::json_v2::GenericBatteryLimit>();
    impl_provider_builder::<Cpus, Vec<crate::persist::CpuJson>, limits_core::json_v2::GenericCpusLimit>();
    impl_provider_builder::<Gpu, crate::persist::GpuJson, limits_core::json_v2::GenericGpuLimit>();
}
