use crate::persist::CpuJson;
use limits_core::json_v2::GenericCpuLimit;

// similar to crate::settings::ProviderBuilder<CpuJson, GenericCpuLimit>
pub trait FromGenericCpuInfo {
    fn from_limits(cpu_index: usize, limits: GenericCpuLimit) -> Self;

    fn from_json_and_limits(
        other: CpuJson,
        version: u64,
        cpu_index: usize,
        limits: GenericCpuLimit,
    ) -> Self;
}
