use std::fmt;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum AcceleratorType {
  #[serde(rename = "NVIDIA")]
  Nvidia,
  #[serde(rename = "Neuron")]
  Neuron,
  #[serde(rename = "None")]
  None,
}

impl std::fmt::Display for AcceleratorType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AcceleratorType::Nvidia => write!(f, "NVIDIA"),
      AcceleratorType::Neuron => write!(f, "Neuron"),
      AcceleratorType::None => write!(f, "None"),
    }
  }
}

#[derive(Debug, EnumIter, PartialEq, Serialize, Deserialize)]
pub enum ScalingType {
  #[serde(rename = "karpenter")]
  Karpenter,
  #[serde(rename = "cluster-autoscaler")]
  ClusterAutoscaler,
  #[serde(rename = "None")]
  None,
}

impl std::convert::From<&str> for ScalingType {
  fn from(s: &str) -> Self {
    match s {
      "karpenter" => ScalingType::Karpenter,
      "cluster-autoscaler" => ScalingType::ClusterAutoscaler,
      _ => ScalingType::None,
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CpuArch {
  X8664,
  Arm64,
}

impl fmt::Display for CpuArch {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CpuArch::X8664 => write!(f, "x86-64"),
      CpuArch::Arm64 => write!(f, "arm64"),
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ReservationType {
  #[serde(rename = "ODCR")]
  OnDemandCapacityReservation,
  #[serde(rename = "CBR")]
  MlCapacityBlockReservation,
  #[serde(rename = "None")]
  None,
}

impl std::convert::From<&str> for ReservationType {
  fn from(s: &str) -> Self {
    match s {
      "On-demand capacity reservation" => ReservationType::OnDemandCapacityReservation,
      "ML capacity block reservation" => ReservationType::MlCapacityBlockReservation,
      _ => ReservationType::None,
    }
  }
}
