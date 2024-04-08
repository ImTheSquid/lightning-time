#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::{str::FromStr, sync::OnceLock};

use chrono::{NaiveTime, Timelike};
#[cfg(feature = "std")]
use regex::Regex;
use thiserror_no_std::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightningTime {
    pub bolts: u8,
    pub bolt_colors: LightningBaseColors,
    pub zaps: u8,
    pub zap_colors: LightningBaseColors,
    pub sparks: u8,
    pub spark_colors: LightningBaseColors,
    pub charges: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightningBaseColors(pub u8, pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightningTimeColors {
    pub bolt: palette::Srgb<u8>,
    pub zap: palette::Srgb<u8>,
    pub spark: palette::Srgb<u8>,
}

impl Default for LightningTime {
    fn default() -> Self {
        Self {
            bolts: 0,
            bolt_colors: LightningBaseColors(161, 0),
            zaps: 0,
            zap_colors: LightningBaseColors(50, 214),
            sparks: 0,
            spark_colors: LightningBaseColors(246, 133),
            charges: 0,
        }
    }
}

impl LightningTime {
    pub fn new(bolts: u8, zaps: u8, sparks: u8, charges: u8) -> Self {
        Self {
            bolts,
            zaps,
            sparks,
            charges,
            ..Default::default()
        }
    }

    pub fn colors(&self) -> LightningTimeColors {
        LightningTimeColors {
            bolt: palette::Srgb::new(
                self.bolts * 16 + self.zaps,
                self.bolt_colors.0,
                self.bolt_colors.1,
            ),
            zap: palette::Srgb::new(
                self.zap_colors.0,
                self.zaps * 16 + self.sparks,
                self.zap_colors.1,
            ),
            spark: palette::Srgb::new(
                self.spark_colors.0,
                self.spark_colors.1,
                self.sparks * 16 + self.charges,
            ),
        }
    }

    #[cfg(feature = "std")]
    pub fn to_stripped_string(&self) -> String {
        format!("{:x}~{:x}~{:x}", self.bolts, self.zaps, self.sparks)
    }
}

const MILLIS_PER_CHARGE: f64 = 86_400_000.0 / 65_536.0; // Div by 16^4

impl From<NaiveTime> for LightningTime {
    fn from(value: NaiveTime) -> Self {
        let millis = 1_000. * 60. * 60. * value.hour() as f64
            + 1_000. * 60. * value.minute() as f64
            + 1_000. * value.second() as f64
            + value.nanosecond() as f64 / 1.0e6;

        let total_charges = millis / MILLIS_PER_CHARGE;
        let total_sparks = total_charges / 16.;
        let total_zaps = total_sparks / 16.;
        let total_bolts = total_zaps / 16.;

        #[cfg(feature = "std")]
        {
            LightningTime {
                bolts: (total_bolts.floor() % 16.) as u8,
                sparks: (total_sparks.floor() % 16.) as u8,
                zaps: (total_zaps.floor() % 16.) as u8,
                charges: (total_charges.floor() % 16.) as u8,
                ..Default::default()
            }
        }

        #[cfg(not(feature = "std"))]
        {
            use libm::floor;
            LightningTime {
                bolts: (floor(total_bolts) % 16.) as u8,
                sparks: (floor(total_sparks) % 16.) as u8,
                zaps: (floor(total_zaps) % 16.) as u8,
                charges: (floor(total_charges) % 16.) as u8,
                ..Default::default()
            }
        }
    }
}

#[cfg(feature = "std")]
static RE: OnceLock<Regex> = OnceLock::new();

#[cfg(feature = "std")]
impl FromStr for LightningTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = RE.get_or_init(|| {
            Regex::new(r"(?P<bolt>[[:xdigit:]])~(?P<spark>[[:xdigit:]])~(?P<zap>[[:xdigit:]])(?:\|(?P<charge>[[:xdigit:]]))?").unwrap()
        });

        let caps = re.captures(s);
        match caps {
            Some(caps) => {
                if caps.len() < 4 {
                    return Err(Error::InvalidConversion);
                }
                Ok(LightningTime {
                    bolts: u8::from_str_radix(caps.name("bolt").unwrap().as_str(), 16).unwrap(),
                    zaps: u8::from_str_radix(caps.name("zap").unwrap().as_str(), 16).unwrap(),
                    sparks: u8::from_str_radix(caps.name("spark").unwrap().as_str(), 16).unwrap(),
                    charges: caps
                        .name("charge")
                        .map(|c| u8::from_str_radix(c.as_str(), 16).unwrap())
                        .unwrap_or(0),
                    ..Default::default()
                })
            }
            None => Err(Error::InvalidConversion),
        }
    }
}

impl core::fmt::Display for LightningTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{:x}~{:x}~{:x}|{:x}",
            self.bolts, self.zaps, self.sparks, self.charges
        ))
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum Error {
    #[error("Invalid conversion")]
    InvalidConversion,
}

impl TryInto<NaiveTime> for LightningTime {
    type Error = Error;

    fn try_into(self) -> Result<NaiveTime, Self::Error> {
        let elapsed: usize =
            ((self.bolts as usize * 16 + self.zaps as usize) * 16 + self.sparks as usize) * 16
                + self.charges as usize;

        let millis = elapsed as f64 * MILLIS_PER_CHARGE;

        let seconds = millis / 1000.;
        let leftover_millis = millis % 1000.;

        NaiveTime::from_num_seconds_from_midnight_opt(
            seconds as u32,
            (leftover_millis * 1.0e6) as u32,
        )
        .ok_or(Error::InvalidConversion)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::{NaiveTime, Timelike};
    use palette::Srgb;

    use crate::{Error, LightningTime, LightningTimeColors};

    #[test]
    fn convert_to_lightning() {
        let real = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let lightning = LightningTime::from(real);
        assert_eq!(
            lightning,
            LightningTime {
                bolts: 0x8,
                ..Default::default()
            }
        );

        assert_eq!(lightning.to_string(), "8~0~0|0");
        assert_eq!(lightning.to_stripped_string(), "8~0~0");
        assert_eq!(
            lightning.colors(),
            LightningTimeColors {
                bolt: Srgb::new(0x80, 0xa1, 0x00),
                zap: Srgb::new(0x32, 0x00, 0xd6),
                spark: Srgb::new(0xf6, 0x85, 0x00),
            }
        );
    }

    #[test]
    fn parse() {
        assert!(LightningTime::from_str("f~3~a|8").is_ok());
        assert!(LightningTime::from_str("f~3~a").is_ok());
        assert!(LightningTime::from_str("f~~|").is_err());
    }

    #[test]
    fn convert_to_real() {
        let lightning = LightningTime {
            bolts: 0x8,
            ..Default::default()
        };

        let naive: Result<NaiveTime, Error> = lightning.try_into();

        assert!(naive.is_ok());
        assert_eq!(naive.unwrap(), NaiveTime::from_hms_opt(12, 0, 0).unwrap());

        let lightning = LightningTime {
            bolts: 0x8,
            charges: 0xa,
            ..Default::default()
        };

        let naive: Result<NaiveTime, Error> = lightning.try_into();

        assert!(naive.is_ok());
        // Floating point is not fun
        assert_eq!(
            naive.unwrap().second(),
            NaiveTime::from_hms_opt(12, 0, 13).unwrap().second()
        );
    }
}
