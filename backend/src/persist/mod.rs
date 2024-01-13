mod battery;
mod cpu;
mod driver;
mod error;
mod file;
mod general;
mod gpu;

pub use battery::{BatteryEventJson, BatteryJson};
pub use cpu::CpuJson;
pub use driver::DriverJson;
pub use file::FileJson;
pub use general::{MinMaxJson, SettingsJson};
pub use gpu::GpuJson;

pub use error::SerdeError;

pub const LATEST_VERSION: u64 = 0;
