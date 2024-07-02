use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use super::{
  ami,
  instance::{self, InstanceInfo},
};

/// Returns map of instance type name => instance type info
fn get_instance_types<'a>(
  cpu_arch: &CpuArch,
  enable_efa: bool,
  accelerator: &AcceleratorType,
  reservation: &ReservationType,
) -> BTreeMap<&'a str, &'a InstanceInfo<'a>> {
  instance::INSTANCE_TYPES
    .iter()
    .filter(|i| {
      i.cpu_arch == cpu_arch.to_string()
        && if enable_efa { i.efa_supported } else { true }
        && if accelerator == &AcceleratorType::Nvidia {
          i.nvidia_gpu_supported
        } else if accelerator == &AcceleratorType::Neuron {
          i.neuron_supported
        } else {
          true
        }
        && if reservation == &ReservationType::MlCapacityBlockReservation {
          i.cbr_supported
        } else {
          true
        }
    })
    .map(|i| (i.instance_type, i))
    .collect::<BTreeMap<&'a str, &InstanceInfo<'a>>>()
}

pub fn get_instance_type_names<'a>(
  cpu_arch: &CpuArch,
  enable_efa: bool,
  accelerator: &AcceleratorType,
  reservation: &ReservationType,
) -> Vec<&'a str> {
  get_instance_types(cpu_arch, enable_efa, accelerator, reservation)
    .keys()
    .copied()
    .collect()
}

pub fn instance_storage_supported(instance_types: &[String], ami_type: &ami::AmiType) -> bool {
  let instance_types_support = instance::INSTANCE_TYPES
    .iter()
    .filter(|instance| instance_types.contains(&instance.instance_type.to_string()))
    .map(|instance| instance.instance_storage_supported)
    .all(|f| f);

  match ami_type {
    ami::AmiType::Al2023Arm64Standard
    | ami::AmiType::Al2023X8664Standard
    | ami::AmiType::Al2Arm64
    | ami::AmiType::Al2X8664
    | ami::AmiType::Al2X8664Gpu => instance_types_support,
    _ => false,
  }
}

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

pub fn get_scaling_types<'a>(reservation: &ReservationType) -> Vec<&'a str> {
  match reservation {
    ReservationType::None => vec!["karpenter", "cluster-autoscaler", "None"],
    _ => vec!["cluster-autoscaler", "None"],
  }
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

pub fn get_reservation_types<'a>(accelerator: &AcceleratorType) -> Vec<&'a str> {
  match accelerator {
    AcceleratorType::Nvidia => vec![
      "None",
      "On-demand capacity reservation",
      "ML capacity block reservation",
    ],
    _ => vec!["None", "On-demand capacity reservation"],
  }
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
