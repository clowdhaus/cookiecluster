use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

use super::{ami, instance};

/// Filter and return available instance types by name
pub fn get_instance_type_names<'a>(
  cpu_arch: &CpuArch,
  require_efa: bool,
  accelerator: &AcceleratorType,
  reservation: &ReservationType,
) -> Vec<&'a str> {
  instance::INSTANCE_TYPES
    .iter()
    .filter(|i| i.cpu_arch == cpu_arch.to_string())
    .filter(|i| {
      if reservation == &ReservationType::MlCapacityBlockReservation {
        i.cbr_supported
      } else {
        true
      }
    })
    .filter(|i| {
      if accelerator == &AcceleratorType::Nvidia {
        i.nvidia_gpu_supported
      } else if accelerator == &AcceleratorType::Neuron {
        i.neuron_supported
      } else if accelerator == &AcceleratorType::None {
        !i.nvidia_gpu_supported && !i.neuron_supported
      } else {
        false // no-op
      }
    })
    .filter(|i| {
      // If `require_efa` is true, only return instances that support EFA
      // otherwise, do not filter out any instances based on EFA support
      if require_efa { i.efa_supported } else { true }
    })
    .map(|i| i.instance_type)
    .collect::<Vec<&str>>()
}

pub fn limit_instances_selected(
  reservation: &ReservationType,
  require_efa: bool,
  instance_type_names: Vec<&str>,
  mut instance_idxs: Vec<usize>,
) -> Result<Vec<String>> {
  if instance_idxs.is_empty() {
    bail!("At least one instance type needs to be selected");
  }

  // There are two scenarios where only a single instance type should be specified:
  // 1. EC2 capacity reservation(s)
  // 2. When using EFA
  if reservation != &ReservationType::None || require_efa {
    instance_idxs = vec![instance_idxs.last().unwrap().to_owned()];
  }

  let instance_types = instance_idxs
    .iter()
    .map(|&i| instance_type_names[i].to_string())
    .collect::<Vec<String>>();

  Ok(instance_types)
}

pub fn instance_storage_supported(instance_types: &[String], ami_type: &ami::AmiType) -> bool {
  let instance_types_support = instance::INSTANCE_TYPES
    .iter()
    .filter(|instance| instance_types.contains(&instance.instance_type.to_string()))
    .all(|instance| instance.instance_storage_supported);

  match ami_type {
    ami::AmiType::AL2023_ARM_64_STANDARD
    | ami::AmiType::AL2023_x86_64_STANDARD
    | ami::AmiType::AL2023_x86_64_NVIDIA
    | ami::AmiType::AL2023_x86_64_NEURON => instance_types_support,
    _ => false,
  }
}

#[derive(Debug, Display, EnumIter, EnumString, IntoStaticStr, PartialEq, Serialize, Deserialize)]
pub enum AcceleratorType {
  None,
  #[serde(rename = "NVIDIA")]
  #[strum(serialize = "NVIDIA")]
  Nvidia,
  Neuron,
}

impl AcceleratorType {
  #[cfg(not(tarpaulin_include))]
  pub fn from_idx(idx: usize) -> AcceleratorType {
    AcceleratorType::iter().nth(idx).unwrap()
  }
}

#[inline]
#[cfg(not(tarpaulin_include))]
pub fn get_accelerator_types() -> Vec<AcceleratorType> {
  AcceleratorType::iter().collect()
}

#[derive(Debug, Display, EnumIter, EnumString, IntoStaticStr, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScalingType {
  #[strum(serialize = "EKS Auto Mode")]
  AutoMode,
  Karpenter,
  #[strum(serialize = "cluster-autoscaler")]
  ClusterAutoscaler,
  None,
}

impl ScalingType {
  #[cfg(not(tarpaulin_include))]
  pub fn from_idx(idx: usize) -> ScalingType {
    ScalingType::iter().nth(idx).unwrap()
  }
}

pub fn get_compute_scaling_types(reservation: &ReservationType) -> Vec<ScalingType> {
  match reservation {
    ReservationType::None => ScalingType::iter().collect(),
    ReservationType::OnDemandCapacityReservation => {
      ScalingType::iter().filter(|s| s != &ScalingType::AutoMode).collect()
    }
    _ => ScalingType::iter()
      .filter(|s| s != &ScalingType::AutoMode && s != &ScalingType::Karpenter)
      .collect(),
  }
}

#[derive(Debug, Display, EnumIter, EnumString, IntoStaticStr, PartialEq, Serialize, Deserialize)]
pub enum CpuArch {
  #[strum(serialize = "x86-64")]
  X8664,
  #[strum(serialize = "arm64")]
  Arm64,
}

impl CpuArch {
  #[cfg(not(tarpaulin_include))]
  pub fn from_idx(idx: usize) -> CpuArch {
    CpuArch::iter().nth(idx).unwrap()
  }
}

#[derive(
  Debug, Display, EnumIter, EnumString, IntoStaticStr, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize,
)]
pub enum ReservationType {
  None,
  #[strum(serialize = "On-demand capacity reservation")]
  OnDemandCapacityReservation,
  #[strum(serialize = "ML capacity block reservation")]
  MlCapacityBlockReservation,
}

impl ReservationType {
  #[cfg(not(tarpaulin_include))]
  pub fn from_idx(idx: usize) -> ReservationType {
    ReservationType::iter().nth(idx).unwrap()
  }
}

pub fn get_reservation_types(accelerator: &AcceleratorType) -> Vec<ReservationType> {
  match accelerator {
    AcceleratorType::Nvidia | AcceleratorType::Neuron => ReservationType::iter().collect(),
    _ => ReservationType::iter()
      .filter(|r| r != &ReservationType::MlCapacityBlockReservation)
      .collect(),
  }
}

#[cfg(test)]
mod tests {
  use rstest::*;

  use super::*;

  #[rstest]
  #[case(&CpuArch::X8664, false, &AcceleratorType::None, &ReservationType::None, vec!["m5.large", "c5.large", "r5.large"], vec!["p4d.24xlarge", "p5.48xlarge"])]
  #[case(&CpuArch::Arm64, false, &AcceleratorType::None, &ReservationType::None, vec!["m7g.large", "c7g.large", "r7g.large"], vec!["p4d.24xlarge", "p5.48xlarge"])]
  #[case(&CpuArch::X8664, true, &AcceleratorType::Nvidia, &ReservationType::None, vec!["g5.24xlarge", "g6e.24xlarge", "p5.48xlarge"], vec!["m5.large", "c5.large", "r5.large"])]
  #[case(&CpuArch::X8664, false, &AcceleratorType::Nvidia, &ReservationType::None, vec!["g5.8xlarge", "g6e.24xlarge", "p5.48xlarge"], vec!["m5.large", "c5.large", "r5.large"])]
  #[case(&CpuArch::X8664, true, &AcceleratorType::Nvidia, &ReservationType::MlCapacityBlockReservation, vec!["p4d.24xlarge", "p5.48xlarge", "p5en.48xlarge"], vec!["m5.large", "c5.large", "r5.large"])]
  #[case(&CpuArch::X8664, true, &AcceleratorType::Neuron, &ReservationType::None, vec!["trn1.32xlarge", "trn1n.32xlarge", "trn2.48xlarge"], vec!["m5.large", "c5.large", "r5.large", "p5.48xlarge"])]
  #[case(&CpuArch::X8664, false, &AcceleratorType::Neuron, &ReservationType::None, vec!["inf2.8xlarge", "inf2.48xlarge", "trn1.2xlarge"], vec!["m5.large", "c5.large", "r5.large", "p5.48xlarge"])]
  #[case(&CpuArch::X8664, true, &AcceleratorType::Neuron, &ReservationType::MlCapacityBlockReservation, vec!["trn1.32xlarge", "trn2.48xlarge"], vec!["p4d.24xlarge", "p5.48xlarge", "p5en.48xlarge"])]
  fn test_get_instance_type_names(
    #[case] cpu_arch: &CpuArch,
    #[case] require_efa: bool,
    #[case] accelerator: &AcceleratorType,
    #[case] reservation: &ReservationType,
    #[case] expected: Vec<&str>,
    #[case] not_expected: Vec<&str>,
  ) {
    let instance_types = get_instance_type_names(cpu_arch, require_efa, accelerator, reservation);
    assert!(expected.iter().all(|item| instance_types.contains(item)));
    assert!(not_expected.iter().all(|item| !instance_types.contains(item)));
  }

  #[rstest]
  #[case(&[String::from("t3.micro")], &ami::AmiType::AL2023_x86_64_STANDARD, false)]
  #[case(&[String::from("p4d.24xlarge")], &ami::AmiType::AL2023_x86_64_NVIDIA, true)]
  #[case(&[String::from("p4d.24xlarge"), String::from("p5.48xlarge")], &ami::AmiType::AL2023_x86_64_NVIDIA, true)]
  #[case(&[String::from("p4d.24xlarge"), String::from("t3.micro")], &ami::AmiType::AL2023_x86_64_NVIDIA, false)]
  #[case(&[String::from("p4d.24xlarge"), String::from("p5.48xlarge")], &ami::AmiType::AL2023_x86_64_NVIDIA, true)]
  #[case(&[String::from("p4d.24xlarge"), String::from("p5.48xlarge")], &ami::AmiType::BOTTLEROCKET_x86_64_NVIDIA, false)]
  fn test_instance_storage_supported(
    #[case] instance_types: &[String],
    #[case] ami_type: &ami::AmiType,
    #[case] expected: bool,
  ) {
    let supported = instance_storage_supported(instance_types, ami_type);
    assert_eq!(supported, expected);
  }

  #[rstest]
  #[case(&ReservationType::None, vec![ScalingType::AutoMode, ScalingType::Karpenter, ScalingType::ClusterAutoscaler, ScalingType::None])]
  #[case(&ReservationType::OnDemandCapacityReservation, vec![ScalingType::Karpenter, ScalingType::ClusterAutoscaler, ScalingType::None])]
  #[case(&ReservationType::MlCapacityBlockReservation, vec![ScalingType::ClusterAutoscaler, ScalingType::None])]
  fn test_get_compute_scaling_types(#[case] reservation: &ReservationType, #[case] expected: Vec<ScalingType>) {
    let scaling_types = get_compute_scaling_types(reservation);
    assert_eq!(scaling_types, expected);
  }

  #[rstest]
  #[case(&AcceleratorType::None, vec![ReservationType::None, ReservationType::OnDemandCapacityReservation])]
  #[case(&AcceleratorType::Nvidia, vec![ReservationType::None, ReservationType::OnDemandCapacityReservation, ReservationType::MlCapacityBlockReservation])]
  #[case(&AcceleratorType::Neuron, vec![ReservationType::None, ReservationType::OnDemandCapacityReservation, ReservationType::MlCapacityBlockReservation])]
  fn test_get_reservation_types(#[case] accelerator: &AcceleratorType, #[case] mut expected: Vec<ReservationType>) {
    let mut reservation_types = get_reservation_types(accelerator);
    reservation_types.sort();
    expected.sort();
    assert_eq!(reservation_types, expected);
  }

  #[rstest]
  #[case(&ReservationType::None, true, vec!["t2.micro", "t3.micro", "t3a.micro"], vec!["t3a.micro".to_string()])]
  fn test_limit_instances_selected(
    #[case] reservation: &ReservationType,
    #[case] require_efa: bool,
    #[case] instance_type_names: Vec<&str>,
    #[case] expected: Vec<String>,
  ) {
    let instance_idxs = vec![0, 1, 2];
    let instance_types =
      limit_instances_selected(reservation, require_efa, instance_type_names, instance_idxs).unwrap();
    assert_eq!(instance_types, expected);
  }

  #[test]
  fn snapshot_x86_64() {
    let standard_x86_64 =
      get_instance_type_names(&CpuArch::X8664, false, &AcceleratorType::None, &ReservationType::None);
    insta::assert_debug_snapshot!(standard_x86_64);
  }

  #[test]
  fn snapshot_arm64() {
    let standard_arm64 =
      get_instance_type_names(&CpuArch::Arm64, false, &AcceleratorType::None, &ReservationType::None);
    insta::assert_debug_snapshot!(standard_arm64);
  }

  #[test]
  fn snapshot_nvidia_x86_64() {
    let nvidia_x86_64 =
      get_instance_type_names(&CpuArch::X8664, false, &AcceleratorType::Nvidia, &ReservationType::None);
    insta::assert_debug_snapshot!(nvidia_x86_64);
  }

  #[test]
  fn snapshot_nvidia_x86_64_efa() {
    let nvidia_x86_64_efa =
      get_instance_type_names(&CpuArch::X8664, true, &AcceleratorType::Nvidia, &ReservationType::None);
    insta::assert_debug_snapshot!(nvidia_x86_64_efa);
  }

  #[test]
  fn snapshot_nvidia_x86_64_ml_cbr() {
    let nvidia_x86_64_ml_cbr = get_instance_type_names(
      &CpuArch::X8664,
      false,
      &AcceleratorType::Nvidia,
      &ReservationType::MlCapacityBlockReservation,
    );
    insta::assert_debug_snapshot!(nvidia_x86_64_ml_cbr);
  }

  #[test]
  fn snapshot_nvidia_x86_64_odcr() {
    let nvidia_x86_64_odcr = get_instance_type_names(
      &CpuArch::X8664,
      false,
      &AcceleratorType::Nvidia,
      &ReservationType::OnDemandCapacityReservation,
    );
    insta::assert_debug_snapshot!(nvidia_x86_64_odcr);
  }

  #[test]
  fn snapshot_neuron_x86_64() {
    let neuron_x86_64 =
      get_instance_type_names(&CpuArch::X8664, false, &AcceleratorType::Neuron, &ReservationType::None);
    insta::assert_debug_snapshot!(neuron_x86_64);
  }

  #[test]
  fn snapshot_neuron_x86_64_efa() {
    let neuron_x86_64_efa =
      get_instance_type_names(&CpuArch::X8664, true, &AcceleratorType::Neuron, &ReservationType::None);
    insta::assert_debug_snapshot!(neuron_x86_64_efa);
  }

  #[test]
  fn snapshot_neuron_x86_64_ml_cbr() {
    let neuron_x86_64_ml_cbr = get_instance_type_names(
      &CpuArch::X8664,
      false,
      &AcceleratorType::Neuron,
      &ReservationType::MlCapacityBlockReservation,
    );
    insta::assert_debug_snapshot!(neuron_x86_64_ml_cbr);
  }

  #[test]
  fn snapshot_neuron_x86_64_odcr() {
    let neuron_x86_64_odcr = get_instance_type_names(
      &CpuArch::X8664,
      false,
      &AcceleratorType::Neuron,
      &ReservationType::OnDemandCapacityReservation,
    );
    insta::assert_debug_snapshot!(neuron_x86_64_odcr);
  }

  #[test]
  fn snapshot_x86_64_efa() {
    let efa_x86_64 = get_instance_type_names(&CpuArch::X8664, true, &AcceleratorType::None, &ReservationType::None);
    insta::assert_debug_snapshot!(efa_x86_64);
  }

  #[test]
  fn snapshot_arm64_efa() {
    let efa_arm64 = get_instance_type_names(&CpuArch::Arm64, true, &AcceleratorType::None, &ReservationType::None);
    insta::assert_debug_snapshot!(efa_arm64);
  }
}
