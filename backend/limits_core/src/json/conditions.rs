use serde::{Deserialize, Serialize};

/// Conditions under which a config applies
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Conditions {
    /// Regex pattern for dmidecode output
    pub dmi: Option<String>,
    /// Regex pattern for /proc/cpuinfo reading
    pub cpuinfo: Option<String>,
    /// Regex pattern for /etc/os-release reading
    pub os: Option<String>,
    /// Custom command to run, where an exit code of 0 means a successful match
    pub command: Option<String>,
}

impl Conditions {
    pub fn is_empty(&self) -> bool {
        self.dmi.is_none() && self.cpuinfo.is_none() && self.os.is_none() && self.command.is_none()
    }
}