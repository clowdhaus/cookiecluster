use std::fmt;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, PartialEq, Serialize, Deserialize)]
pub enum ClusterVersion {
  #[serde(rename = "1.30")]
  K130,
  #[serde(rename = "1.29")]
  K129,
  #[serde(rename = "1.28")]
  K128,
  #[serde(rename = "1.27")]
  K127,
  #[serde(rename = "1.26")]
  K126,
  #[serde(rename = "1.25")]
  K125,
}

impl std::fmt::Display for ClusterVersion {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ClusterVersion::K125 => write!(f, "1.25"),
      ClusterVersion::K126 => write!(f, "1.26"),
      ClusterVersion::K127 => write!(f, "1.27"),
      ClusterVersion::K128 => write!(f, "1.28"),
      ClusterVersion::K129 => write!(f, "1.29"),
      ClusterVersion::K130 => write!(f, "1.30"),
    }
  }
}

impl std::convert::From<&str> for ClusterVersion {
  fn from(s: &str) -> Self {
    match s {
      "1.25" => ClusterVersion::K125,
      "1.26" => ClusterVersion::K126,
      "1.27" => ClusterVersion::K127,
      "1.28" => ClusterVersion::K128,
      "1.29" => ClusterVersion::K129,
      _ => ClusterVersion::K130,
    }
  }
}
