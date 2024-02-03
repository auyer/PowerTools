pub const PORT: u16 = 44443;

pub const PACKAGE_NAME: &'static str = env!("CARGO_PKG_NAME");
pub const PACKAGE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_SETTINGS_FILE: &str = "default_settings.ron";
pub const DEFAULT_SETTINGS_NAME: &str = "Main";
pub const DEFAULT_SETTINGS_VARIANT_NAME: &str = "Primary";

pub const LIMITS_FILE: &str = "limits_cache.ron";
pub const LIMITS_OVERRIDE_FILE: &str = "limits_override.ron";

pub const WEB_SETTINGS_CACHE: &str = "store_cache.ron";

pub const MESSAGE_SEEN_ID_FILE: &str = "seen_message.bin";
