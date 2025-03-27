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
