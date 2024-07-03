use std::{collections::BTreeMap, fmt};

use anyhow::{bail, Result};
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

pub fn limit_instances_selected(
  reservation: &ReservationType,
  enable_efa: bool,
  instance_type_names: Vec<&str>,
  mut instance_idxs: Vec<usize>,
) -> Result<Vec<String>> {
  // There are two scenarios where only a single instance type should be specified:
  // 1. EC2 capacity reservation(s)
  // 2. When using EFA
  if reservation != &ReservationType::None || enable_efa {
    instance_idxs = vec![instance_idxs.last().unwrap().to_owned()];
  }

  let instance_types = instance_idxs
    .iter()
    .map(|&i| instance_type_names[i].to_string())
    .collect::<Vec<String>>();

  if instance_types.is_empty() {
    bail!("At least one instance type needs to be selected");
  }

  Ok(instance_types)
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

#[cfg(test)]
mod tests {

  use rstest::*;

  use super::*;

  #[rstest]
  #[case(&[String::from("t2.micro")], &ami::AmiType::Al2X8664Gpu, false)]
  #[case(&[String::from("p4d.24xlarge")], &ami::AmiType::Al2X8664Gpu, true)]
  #[case(&[String::from("p4d.24xlarge"), String::from("p5.48xlarge")], &ami::AmiType::Al2X8664Gpu, true)]
  #[case(&[String::from("p4d.24xlarge"), String::from("t2.micro")], &ami::AmiType::Al2X8664Gpu, false)]
  #[case(&[String::from("p4d.24xlarge"), String::from("p5.48xlarge")], &ami::AmiType::Al2X8664, true)]
  #[case(&[String::from("p4d.24xlarge"), String::from("p5.48xlarge")], &ami::AmiType::BottlerocketX8664Nvidia, false)]
  fn test_instance_storage_supported(
    #[case] instance_types: &[String],
    #[case] ami_type: &ami::AmiType,
    #[case] expected: bool,
  ) {
    let supported = instance_storage_supported(instance_types, ami_type);
    assert_eq!(supported, expected);
  }

  #[rstest]
  #[case(&ReservationType::None, true, vec!["t2.micro", "t3.micro", "t3a.micro"], vec!["t3a.micro".to_string()])]
  fn test_limit_instances_selected(
    #[case] reservation: &ReservationType,
    #[case] enable_efa: bool,
    #[case] instance_type_names: Vec<&str>,
    #[case] expected: Vec<String>,
  ) {
    let instance_idxs = vec![0, 1, 2];
    let instance_types = limit_instances_selected(reservation, enable_efa, instance_type_names, instance_idxs).unwrap();
    assert_eq!(instance_types, expected);
  }

  #[test]
  fn snapshot_instance_types() {
    let standard_x86_64 = get_instance_types(&CpuArch::X8664, false, &AcceleratorType::None, &ReservationType::None);
    insta::assert_debug_snapshot!(standard_x86_64);

    let standard_arm64 = get_instance_types(&CpuArch::Arm64, false, &AcceleratorType::None, &ReservationType::None);
    insta::assert_debug_snapshot!(standard_arm64);

    let nvidia_x86_64 = get_instance_types(&CpuArch::X8664, false, &AcceleratorType::Nvidia, &ReservationType::None);
    insta::assert_debug_snapshot!(nvidia_x86_64);

    let efa_nvidia_x86_64 = get_instance_types(&CpuArch::X8664, true, &AcceleratorType::Nvidia, &ReservationType::None);
    insta::assert_debug_snapshot!(efa_nvidia_x86_64);

    let neuron_x86_64 = get_instance_types(&CpuArch::X8664, false, &AcceleratorType::Neuron, &ReservationType::None);
    insta::assert_debug_snapshot!(neuron_x86_64);

    let efa_neuron_x86_64 = get_instance_types(&CpuArch::X8664, true, &AcceleratorType::Neuron, &ReservationType::None);
    insta::assert_debug_snapshot!(efa_neuron_x86_64);

    let nvidia_cbr_reservation = get_instance_types(
      &CpuArch::X8664,
      false,
      &AcceleratorType::Nvidia,
      &ReservationType::MlCapacityBlockReservation,
    );
    insta::assert_debug_snapshot!(nvidia_cbr_reservation);

    let efa_x86_64 = get_instance_types(&CpuArch::X8664, true, &AcceleratorType::None, &ReservationType::None);
    insta::assert_debug_snapshot!(efa_x86_64);

    let efa_arm64 = get_instance_types(&CpuArch::Arm64, true, &AcceleratorType::None, &ReservationType::None);
    insta::assert_debug_snapshot!(efa_arm64);
  }
}
