#![allow(dead_code)]

pub const JUPITER_HWMON_NAME: &'static str = "jupiter";
pub const STEAMDECK_HWMON_NAME: &'static str = "steamdeck_hwmon";
pub const GPU_HWMON_NAME: &'static str = "amdgpu";

pub fn range_min_or_fallback<I: Copy>(range: &Option<limits_core::json_v2::RangeLimit<I>>, fallback: I) -> I {
    range.and_then(|lim| lim.min).unwrap_or(fallback)
}

pub fn range_max_or_fallback<I: Copy>(range: &Option<limits_core::json_v2::RangeLimit<I>>, fallback: I) -> I {
    range.and_then(|lim| lim.max).unwrap_or(fallback)
}

pub fn card_also_has(card: &dyn sysfuss::SysEntity, extensions: &'static [&'static str]) -> bool {
    extensions.iter()
        .all(|ext| card.as_ref().join(ext).exists())
}

const THINGS: &[u8] = &[
    1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1,
    1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0,
    0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0,
];

const TIME_UNIT: std::time::Duration = std::time::Duration::from_millis(200);

pub fn flash_led() {
    use smokepatio::ec::ControllerSet;
    let mut ec = smokepatio::ec::unnamed_power::UnnamedPowerEC::new();
    for &code in THINGS {
        let on = code != 0;
        let colour = if on { smokepatio::ec::unnamed_power::StaticColour::Red } else { smokepatio::ec::unnamed_power::StaticColour::Off };
        if let Err(e) = ec.set(colour) {
            log::error!("Thing err: {}", e);
        }
        std::thread::sleep(TIME_UNIT);
    }
    log::debug!("Restoring LED state");
    ec.set(smokepatio::ec::unnamed_power::StaticColour::Off)
        .map_err(|e| log::error!("Failed to restore LED status: {}", e))
        .unwrap();
}
