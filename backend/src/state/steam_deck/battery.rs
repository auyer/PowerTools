#[derive(Debug, Clone)]
pub struct Battery {
    pub charge_rate_set: bool,
    pub charge_mode_set: bool,
    pub charger_state: ChargeState,
    pub charge_limit_set: bool,
}

impl std::default::Default for Battery {
    fn default() -> Self {
        Self {
            charge_rate_set: true,
            charge_mode_set: true,
            charger_state: ChargeState::Unknown,
            charge_limit_set: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChargeState {
    PluggedIn,
    Unplugged,
    Unknown,
}
