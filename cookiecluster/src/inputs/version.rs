use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(Debug, EnumIter, Display, EnumString, IntoStaticStr, PartialEq, Serialize, Deserialize)]
pub enum ClusterVersion {
  #[strum(serialize = "1.32")]
  #[serde(rename = "1.32")]
  K132,
  #[strum(serialize = "1.31")]
  #[serde(rename = "1.31")]
  K131,
  #[strum(serialize = "1.30")]
  #[serde(rename = "1.30")]
  K130,
  #[strum(serialize = "1.29")]
  #[serde(rename = "1.29")]
  K129,
  #[strum(serialize = "1.28")]
  #[serde(rename = "1.28")]
  K128,
  #[strum(serialize = "1.27")]
  #[serde(rename = "1.27")]
  K127,
}

impl ClusterVersion {
  #[cfg(not(tarpaulin_include))]
  pub fn versions() -> Vec<ClusterVersion> {
    ClusterVersion::iter().collect()
  }

  #[cfg(not(tarpaulin_include))]
  pub fn from_idx(idx: usize) -> ClusterVersion {
    ClusterVersion::iter().nth(idx).unwrap()
  }
}
