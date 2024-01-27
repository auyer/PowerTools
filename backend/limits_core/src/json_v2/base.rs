use serde::{Deserialize, Serialize};
use std::default::Default;

/// Base JSON limits information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Base {
    /// System-specific configurations
    pub configs: Vec<super::Config>,
    /// Server messages
    pub messages: Vec<super::DeveloperMessage>,
    /// URL from which to grab the next update
    pub refresh: Option<String>,
}

impl Default for Base {
    fn default() -> Self {
        Base {
            configs: vec![
                super::Config {
                    name: "Steam Deck".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t: AMD Custom APU 0405\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: super::Limits {
                        cpu: super::Limit {
                            provider: super::CpuLimitType::SteamDeck,
                            limits: super::GenericCpusLimit::default_for(super::CpuLimitType::SteamDeck),
                        },
                        gpu: super::Limit {
                            provider: super::GpuLimitType::SteamDeck,
                            limits: super::GenericGpuLimit::default_for(super::GpuLimitType::SteamDeck),
                        },
                        battery: super::Limit {
                            provider: super::BatteryLimitType::SteamDeck,
                            limits: super::GenericBatteryLimit::default_for(super::BatteryLimitType::SteamDeck),
                        },
                    }
                },
                super::Config {
                    name: "Steam Deck OLED".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t: AMD Custom APU 0932\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: super::Limits {
                        cpu: super::Limit {
                            provider: super::CpuLimitType::SteamDeck,
                            limits: super::GenericCpusLimit::default_for(super::CpuLimitType::SteamDeck),
                        },
                        gpu: super::Limit {
                            provider: super::GpuLimitType::SteamDeck,
                            limits: super::GenericGpuLimit::default_for(super::GpuLimitType::SteamDeckOLED),
                        },
                        battery: super::Limit {
                            provider: super::BatteryLimitType::SteamDeck,
                            limits: super::GenericBatteryLimit::default_for(super::BatteryLimitType::SteamDeck),
                        },
                    }
                },
                super::Config {
                    name: "AMD R3 2300U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t+: AMD Ryzen 3 2300U\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: super::Limits {
                        cpu: super::CpuLimit {
                            provider: super::CpuLimitType::GenericAMD,
                            limits: super::GenericCpusLimit {
                                cpus: vec![
                                    super::GenericCpuLimit {
                                        clock_min: Some(super::RangeLimit { min: Some(1000), max: Some(3700) }),
                                        clock_max: Some(super::RangeLimit { min: Some(1000), max: Some(3700) }),
                                        clock_step: Some(100),
                                        skip_resume_reclock: false,
                                    }; 4],
                                global_governors: true,
                            }
                        },
                        gpu: super::GpuLimit {
                            provider: super::GpuLimitType::GenericAMD,
                            limits: super::GenericGpuLimit {
                                fast_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(25_000) }),
                                slow_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(25_000) }),
                                ppt_step: Some(1_000),
                                ppt_divisor: Some(1_000),
                                clock_min: Some(super::RangeLimit { min: Some(400), max: Some(1100) }),
                                clock_max: Some(super::RangeLimit { min: Some(400), max: Some(1100) }),
                                clock_step: Some(100),
                                ..Default::default()
                            }
                        },
                        battery: super::Limit {
                            provider: super::BatteryLimitType::Generic,
                            limits: super::GenericBatteryLimit::default_for(super::BatteryLimitType::Generic),
                        }
                    },
                },
                super::Config {
                    name: "AMD R5 5560U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t+: AMD Ryzen 5 5560U\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: super::Limits {
                        cpu: super::CpuLimit {
                            provider: super::CpuLimitType::GenericAMD,
                            limits: super::GenericCpusLimit {
                                cpus: vec![
                                    super::GenericCpuLimit {
                                        clock_min: Some(super::RangeLimit { min: Some(1000), max: Some(4000) }),
                                        clock_max: Some(super::RangeLimit { min: Some(1000), max: Some(4000) }),
                                        clock_step: Some(100),
                                        skip_resume_reclock: false,
                                    }; 12], // 6 cores with SMTx2
                                global_governors: true,
                            }
                        },
                        gpu: super::GpuLimit {
                            provider: super::GpuLimitType::GenericAMD,
                            limits: super::GenericGpuLimit {
                                fast_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(25_000) }),
                                slow_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(25_000) }),
                                ppt_step: Some(1_000),
                                ppt_divisor: Some(1_000),
                                clock_min: Some(super::RangeLimit { min: Some(400), max: Some(1600) }),
                                clock_max: Some(super::RangeLimit { min: Some(400), max: Some(1600) }),
                                clock_step: Some(100),
                                ..Default::default()
                            }
                        },
                        battery: super::Limit {
                            provider: super::BatteryLimitType::Generic,
                            limits: super::GenericBatteryLimit::default_for(super::BatteryLimitType::Generic),
                        }
                    }
                },
                super::Config {
                    name: "AMD R7 5825U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t+: AMD Ryzen 7 5825U\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: super::Limits {
                        cpu: super::CpuLimit {
                            provider: super::CpuLimitType::GenericAMD,
                            limits: super::GenericCpusLimit {
                                cpus: vec![
                                    super::GenericCpuLimit {
                                        clock_min: Some(super::RangeLimit { min: Some(1000), max: Some(4500) }),
                                        clock_max: Some(super::RangeLimit { min: Some(1000), max: Some(4500) }),
                                        clock_step: Some(100),
                                        skip_resume_reclock: false,
                                    }; 16], // 8 cores with SMTx2
                                global_governors: true,
                            }
                        },
                        gpu: super::GpuLimit {
                            provider: super::GpuLimitType::GenericAMD,
                            limits: super::GenericGpuLimit {
                                fast_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(28_000) }),
                                slow_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(28_000) }),
                                ppt_step: Some(1_000),
                                ppt_divisor: Some(1_000),
                                clock_min: Some(super::RangeLimit { min: Some(400), max: Some(2200) }),
                                clock_max: Some(super::RangeLimit { min: Some(400), max: Some(2200) }),
                                clock_step: Some(100),
                                ..Default::default()
                            }
                        },
                        battery: super::Limit {
                            provider: super::BatteryLimitType::Generic,
                            limits: super::GenericBatteryLimit::default_for(super::BatteryLimitType::Generic),
                        }
                    }
                },
                super::Config {
                    name: "AMD R7 6800U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\t+: AMD Ryzen 7 6800U( with Radeon Graphics)?\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: super::Limits {
                        cpu: super::CpuLimit {
                            provider: super::CpuLimitType::GenericAMD,
                            limits: super::GenericCpusLimit {
                                cpus: vec![
                                    super::GenericCpuLimit {
                                        clock_min: Some(super::RangeLimit { min: Some(1000), max: Some(4700) }),
                                        clock_max: Some(super::RangeLimit { min: Some(1000), max: Some(4700) }),
                                        clock_step: Some(100),
                                        skip_resume_reclock: false,
                                    }; 16], // 8 cores with SMTx2
                                global_governors: true,
                            }
                        },
                        gpu: super::GpuLimit {
                            provider: super::GpuLimitType::GenericAMD,
                            limits: super::GenericGpuLimit {
                                fast_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(28_000) }),
                                slow_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(28_000) }),
                                ppt_step: Some(1_000),
                                ppt_divisor: Some(1_000),
                                clock_min: Some(super::RangeLimit { min: Some(400), max: Some(2200) }),
                                clock_max: Some(super::RangeLimit { min: Some(400), max: Some(2200) }),
                                clock_step: Some(100),
                                ..Default::default()
                            }
                        },
                        battery: super::Limit {
                            provider: super::BatteryLimitType::Generic,
                            limits: super::GenericBatteryLimit::default_for(super::BatteryLimitType::Generic),
                        }
                    }
                },
                super::Config {
                    name: "AMD R7 7840U".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: Some("model name\\s+: AMD Ryzen 7 7840U( w\\/ Radeon  780M Graphics)?\n".to_owned()),
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: super::Limits {
                        cpu: super::CpuLimit {
                            provider: super::CpuLimitType::GenericAMD,
                            limits: super::GenericCpusLimit {
                                cpus: vec![
                                    super::GenericCpuLimit {
                                        clock_min: Some(super::RangeLimit { min: Some(400), max: Some(5100) }),
                                        clock_max: Some(super::RangeLimit { min: Some(400), max: Some(5100) }),
                                        clock_step: Some(100),
                                        skip_resume_reclock: false,
                                    }; 16], // 8 cores with SMTx2
                                global_governors: true,
                            }
                        },
                        gpu: super::GpuLimit {
                            provider: super::GpuLimitType::GenericAMD,
                            limits: super::GenericGpuLimit {
                                fast_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(53_000) }),
                                slow_ppt: Some(super::RangeLimit { min: Some(1_000), max: Some(43_000) }),
                                ppt_step: Some(1_000),
                                ppt_divisor: Some(1_000),
                                clock_min: None,
                                clock_max: None,
                                clock_step: None,
                                ..Default::default()
                            }
                        },
                        battery: super::Limit {
                            provider: super::BatteryLimitType::Generic,
                            limits: super::GenericBatteryLimit::default_for(super::BatteryLimitType::Generic),
                        }
                    }
                },
                super::Config {
                    name: "Fallback".to_owned(),
                    conditions: super::Conditions {
                        dmi: None,
                        cpuinfo: None,
                        os: None,
                        command: None,
                        file_exists: None,
                    },
                    limits: super::Limits {
                        cpu: super::Limit {
                            provider: super::CpuLimitType::Unknown,
                            limits: super::GenericCpusLimit::default_for(super::CpuLimitType::Unknown),
                        },
                        gpu: super::Limit {
                            provider: super::GpuLimitType::Unknown,
                            limits: super::GenericGpuLimit::default_for(super::GpuLimitType::Unknown),
                        },
                        battery: super::Limit {
                            provider: super::BatteryLimitType::Unknown,
                            limits: super::GenericBatteryLimit::default_for(super::BatteryLimitType::Unknown),
                        }
                    }
                }
            ],
            messages: vec![
                super::DeveloperMessage {
                    id: 1,
                    title: "Welcome".to_owned(),
                    body: "Thanks for installing PowerTools! For more information, please check the wiki. For bugs and requests, please create an issue.".to_owned(),
                    url: Some("https://git.ngni.us/NG-SD-Plugins/PowerTools/wiki".to_owned()),
                }
            ],
            refresh: Some("http://limits.ngni.us:45000/powertools/v2".to_owned())
        }
    }
}
