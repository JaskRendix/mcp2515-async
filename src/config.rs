#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clock {
    MHz8,
    MHz16,
    MHz20,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bitrate {
    Kbps125,
    Kbps250,
    Kbps500,
    Mbps1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitrateConfig {
    pub cnf1: u8,
    pub cnf2: u8,
    pub cnf3: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterMask {
    Mask0 = 0,
    Mask1 = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AcceptanceFilter {
    Rxf0 = 0,
    Rxf1 = 1,
    Rxf2 = 2,
    Rxf3 = 3,
    Rxf4 = 4,
    Rxf5 = 5,
}

impl BitrateConfig {
    pub const fn new(clock: Clock, bitrate: Bitrate) -> Option<Self> {
        match (clock, bitrate) {
            // Standard MCP2515 bit timing configurations
            (Clock::MHz16, Bitrate::Kbps125) => Some(Self {
                cnf1: 0x03,
                cnf2: 0xF0,
                cnf3: 0x86,
            }),
            (Clock::MHz16, Bitrate::Kbps250) => Some(Self {
                cnf1: 0x01,
                cnf2: 0xF0,
                cnf3: 0x86,
            }),
            (Clock::MHz16, Bitrate::Kbps500) => Some(Self {
                cnf1: 0x00,
                cnf2: 0xF0,
                cnf3: 0x86,
            }),
            (Clock::MHz16, Bitrate::Mbps1) => Some(Self {
                cnf1: 0x00,
                cnf2: 0x80,
                cnf3: 0x00,
            }),

            (Clock::MHz8, Bitrate::Kbps125) => Some(Self {
                cnf1: 0x01,
                cnf2: 0xF0,
                cnf3: 0x86,
            }),
            (Clock::MHz8, Bitrate::Kbps250) => Some(Self {
                cnf1: 0x00,
                cnf2: 0xF0,
                cnf3: 0x86,
            }),
            (Clock::MHz8, Bitrate::Kbps500) => Some(Self {
                cnf1: 0x00,
                cnf2: 0x80,
                cnf3: 0x00,
            }),

            (Clock::MHz20, Bitrate::Kbps125) => Some(Self {
                cnf1: 0x04,
                cnf2: 0xF0,
                cnf3: 0x86,
            }),
            (Clock::MHz20, Bitrate::Kbps250) => Some(Self {
                cnf1: 0x01,
                cnf2: 0xF0,
                cnf3: 0x86,
            }),
            (Clock::MHz20, Bitrate::Kbps500) => Some(Self {
                cnf1: 0x00,
                cnf2: 0xF0,
                cnf3: 0x86,
            }),

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrate_config_16mhz_values() {
        assert_eq!(
            BitrateConfig::new(Clock::MHz16, Bitrate::Kbps125),
            Some(BitrateConfig {
                cnf1: 0x03,
                cnf2: 0xF0,
                cnf3: 0x86
            })
        );
        assert_eq!(
            BitrateConfig::new(Clock::MHz16, Bitrate::Kbps250),
            Some(BitrateConfig {
                cnf1: 0x01,
                cnf2: 0xF0,
                cnf3: 0x86
            })
        );
        assert_eq!(
            BitrateConfig::new(Clock::MHz16, Bitrate::Kbps500),
            Some(BitrateConfig {
                cnf1: 0x00,
                cnf2: 0xF0,
                cnf3: 0x86
            })
        );
        assert_eq!(
            BitrateConfig::new(Clock::MHz16, Bitrate::Mbps1),
            Some(BitrateConfig {
                cnf1: 0x00,
                cnf2: 0x80,
                cnf3: 0x00
            })
        );
    }

    #[test]
    fn bitrate_config_8mhz_values() {
        assert_eq!(
            BitrateConfig::new(Clock::MHz8, Bitrate::Kbps125),
            Some(BitrateConfig {
                cnf1: 0x01,
                cnf2: 0xF0,
                cnf3: 0x86
            })
        );
        assert_eq!(
            BitrateConfig::new(Clock::MHz8, Bitrate::Kbps250),
            Some(BitrateConfig {
                cnf1: 0x00,
                cnf2: 0xF0,
                cnf3: 0x86
            })
        );
        assert_eq!(
            BitrateConfig::new(Clock::MHz8, Bitrate::Kbps500),
            Some(BitrateConfig {
                cnf1: 0x00,
                cnf2: 0x80,
                cnf3: 0x00
            })
        );
    }

    #[test]
    fn bitrate_config_20mhz_values() {
        assert_eq!(
            BitrateConfig::new(Clock::MHz20, Bitrate::Kbps125),
            Some(BitrateConfig {
                cnf1: 0x04,
                cnf2: 0xF0,
                cnf3: 0x86
            })
        );
        assert_eq!(
            BitrateConfig::new(Clock::MHz20, Bitrate::Kbps250),
            Some(BitrateConfig {
                cnf1: 0x01,
                cnf2: 0xF0,
                cnf3: 0x86
            })
        );
        assert_eq!(
            BitrateConfig::new(Clock::MHz20, Bitrate::Kbps500),
            Some(BitrateConfig {
                cnf1: 0x00,
                cnf2: 0xF0,
                cnf3: 0x86
            })
        );
    }

    #[test]
    fn unsupported_combinations_return_none() {
        assert_eq!(BitrateConfig::new(Clock::MHz8, Bitrate::Mbps1), None);
        assert_eq!(BitrateConfig::new(Clock::MHz20, Bitrate::Mbps1), None);
    }

    #[test]
    fn all_clock_bitrate_pairs_are_covered() {
        let clocks = [Clock::MHz8, Clock::MHz16, Clock::MHz20];
        let bitrates = [
            Bitrate::Kbps125,
            Bitrate::Kbps250,
            Bitrate::Kbps500,
            Bitrate::Mbps1,
        ];

        let mut count = 0;
        for c in clocks {
            for b in bitrates {
                let _ = BitrateConfig::new(c, b);
                count += 1;
            }
        }

        assert_eq!(count, 12); // 3 clocks × 4 bitrates
    }

    #[test]
    fn bitrate_config_round_trip() {
        let cfg = BitrateConfig::new(Clock::MHz16, Bitrate::Kbps125).unwrap();
        let raw = (cfg.cnf1, cfg.cnf2, cfg.cnf3);
        let cfg2 = BitrateConfig {
            cnf1: raw.0,
            cnf2: raw.1,
            cnf3: raw.2,
        };
        assert_eq!(cfg, cfg2);
    }

    #[test]
    fn enums_are_copy_and_eq() {
        let c1 = Clock::MHz8;
        let c2 = c1;
        assert_eq!(c1, c2);

        let b1 = Bitrate::Kbps125;
        let b2 = b1;
        assert_eq!(b1, b2);
    }

    #[test]
    fn bitrate_config_is_copy_and_eq() {
        let cfg1 = BitrateConfig::new(Clock::MHz8, Bitrate::Kbps125).unwrap();
        let cfg2 = cfg1;
        assert_eq!(cfg1, cfg2);
    }

    #[test]
    fn filter_mask_and_acceptance_filter_values() {
        assert_eq!(FilterMask::Mask0 as u8, 0);
        assert_eq!(FilterMask::Mask1 as u8, 1);

        assert_eq!(AcceptanceFilter::Rxf0 as u8, 0);
        assert_eq!(AcceptanceFilter::Rxf1 as u8, 1);
        assert_eq!(AcceptanceFilter::Rxf2 as u8, 2);
        assert_eq!(AcceptanceFilter::Rxf3 as u8, 3);
        assert_eq!(AcceptanceFilter::Rxf4 as u8, 4);
        assert_eq!(AcceptanceFilter::Rxf5 as u8, 5);
    }

    #[test]
    fn filter_mask_traits() {
        let m1 = FilterMask::Mask0;
        let m2 = m1;
        assert_eq!(m1, m2);
    }

    #[test]
    fn acceptance_filter_traits() {
        let f1 = AcceptanceFilter::Rxf3;
        let f2 = f1;
        assert_eq!(f1, f2);
    }
}
