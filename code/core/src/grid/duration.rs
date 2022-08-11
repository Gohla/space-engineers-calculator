use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Serialize, Deserialize, Debug)]
pub struct Duration(f64);

impl Duration {
  #[inline]
  pub fn from_minutes(minutes: f64) -> Self { Self(minutes) }
  #[inline]
  pub fn from_seconds(seconds: f64) -> Self { Self::from_minutes(seconds * SECONDS_TO_MINUTES) }
  #[inline]
  pub fn from_hours(hours: f64) -> Self { Self::from_minutes(hours * HOURS_TO_MINUTES) }
  #[inline]
  pub fn to_f64_and_unit(&self) -> (f64, &str) {
    let d = self.0;
    if d.is_infinite() {
      (d, "")
    } else if d >= MILLENNIUM_TO_MINUTES {
      (d / MILLENNIUM_TO_MINUTES, "millennia")
    } else if d >= YEAR_TO_MINUTES {
      (d / YEAR_TO_MINUTES, "years")
    } else if d >= DAY_TO_MINUTES {
      (d / DAY_TO_MINUTES, "days")
    } else if d >= HOURS_TO_MINUTES {
      (d / HOURS_TO_MINUTES, "hours")
    } else if d <= 1.0 {
      (d / SECONDS_TO_MINUTES, "secs")
    } else {
      (d, Self::DEFAULT_UNIT)
    }
  }

  pub const DEFAULT_UNIT: &'static str = "mins";
}


impl Display for Duration {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let (d, unit) = self.to_f64_and_unit();
    d.fmt(f)?;
    f.write_str(" ")?;
    f.write_str(unit)
  }
}

const MILLENNIUM_TO_MINUTES: f64 = 5.256e+8;
const YEAR_TO_MINUTES: f64 = 525960.0;
const DAY_TO_MINUTES: f64 = 1440.0;
const HOURS_TO_MINUTES: f64 = 60.0;
const SECONDS_TO_MINUTES: f64 = 1.0 / 60.0;
