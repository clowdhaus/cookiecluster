use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

use super::compute::{AcceleratorType, CpuArch};

#[derive(Debug, EnumIter, Display, EnumString, IntoStaticStr, PartialEq, Serialize, Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum AmiType {
  AL2023_ARM_64_STANDARD,
  AL2023_x86_64_STANDARD,
  AL2023_x86_64_NEURON,
  AL2023_x86_64_NVIDIA,
  AL2023_ARM_64_NVIDIA,
  BOTTLEROCKET_ARM_64,
  BOTTLEROCKET_ARM_64_NVIDIA,
  BOTTLEROCKET_x86_64,
  BOTTLEROCKET_x86_64_NVIDIA,
  CUSTOM,
  WINDOWS_CORE_2019_x86_64,
  WINDOWS_CORE_2022_x86_64,
  WINDOWS_FULL_2019_x86_64,
  WINDOWS_FULL_2022_x86_64,
}

impl AmiType {
  #[cfg(not(tarpaulin_include))]
  pub fn from_idx(idx: usize) -> AmiType {
    AmiType::iter().nth(idx).unwrap()
  }
}

pub fn get_ami_types(accelerator: &AcceleratorType, require_efa: bool, cpu_arch: &CpuArch) -> Vec<AmiType> {
  match accelerator {
    AcceleratorType::Nvidia => match (cpu_arch, require_efa) {
      (CpuArch::X8664, true) => vec![AmiType::AL2023_x86_64_NVIDIA, AmiType::BOTTLEROCKET_x86_64_NVIDIA],
      (CpuArch::X8664, false) => vec![AmiType::AL2023_x86_64_NVIDIA, AmiType::BOTTLEROCKET_x86_64_NVIDIA],
      (CpuArch::Arm64, true) => vec![AmiType::AL2023_ARM_64_NVIDIA, AmiType::BOTTLEROCKET_ARM_64_NVIDIA],
      (CpuArch::Arm64, false) => vec![AmiType::AL2023_ARM_64_NVIDIA, AmiType::BOTTLEROCKET_ARM_64_NVIDIA],
    },
    AcceleratorType::Neuron => match (cpu_arch, require_efa) {
      (CpuArch::X8664, true) => vec![AmiType::AL2023_x86_64_NEURON, AmiType::BOTTLEROCKET_x86_64],
      (CpuArch::X8664, false) => vec![AmiType::AL2023_x86_64_NEURON, AmiType::BOTTLEROCKET_x86_64],
      (CpuArch::Arm64, true) => vec![AmiType::BOTTLEROCKET_ARM_64],
      (CpuArch::Arm64, false) => vec![AmiType::BOTTLEROCKET_ARM_64],
    },
    _ => match (cpu_arch, require_efa) {
      (CpuArch::X8664, true) => vec![AmiType::AL2023_x86_64_NVIDIA, AmiType::AL2023_x86_64_NEURON],
      (CpuArch::X8664, false) => vec![
        AmiType::AL2023_x86_64_STANDARD,
        AmiType::BOTTLEROCKET_x86_64,
        AmiType::WINDOWS_CORE_2019_x86_64,
        AmiType::WINDOWS_CORE_2022_x86_64,
        AmiType::WINDOWS_FULL_2019_x86_64,
        AmiType::WINDOWS_FULL_2022_x86_64,
      ],
      (CpuArch::Arm64, true) => vec![AmiType::AL2023_ARM_64_NVIDIA, AmiType::BOTTLEROCKET_ARM_64],
      (CpuArch::Arm64, false) => vec![AmiType::AL2023_ARM_64_STANDARD, AmiType::BOTTLEROCKET_ARM_64],
    },
  }
}

/// Get the default AMI type equivalent based on the AMI type, and CPU architecture
pub fn get_default_ami_type(ami_type: &AmiType, cpu_arch: &CpuArch) -> AmiType {
  match ami_type {
    AmiType::BOTTLEROCKET_x86_64_NVIDIA | AmiType::BOTTLEROCKET_x86_64 => AmiType::BOTTLEROCKET_x86_64,
    AmiType::BOTTLEROCKET_ARM_64_NVIDIA | AmiType::BOTTLEROCKET_ARM_64 => AmiType::BOTTLEROCKET_ARM_64,
    _ => match cpu_arch {
      CpuArch::X8664 => AmiType::AL2023_x86_64_STANDARD,
      CpuArch::Arm64 => AmiType::AL2023_ARM_64_STANDARD,
    },
  }
}

#[cfg(test)]
mod tests {

  use rstest::*;

  use super::*;

  #[rstest]
  #[case(AcceleratorType::Nvidia, false, CpuArch::X8664, vec![AmiType::AL2023_x86_64_NVIDIA, AmiType::BOTTLEROCKET_x86_64_NVIDIA])]
  #[case(AcceleratorType::Nvidia, false, CpuArch::Arm64, vec![AmiType::AL2023_ARM_64_NVIDIA, AmiType::BOTTLEROCKET_ARM_64_NVIDIA])]
  #[case(AcceleratorType::Nvidia, true, CpuArch::X8664, vec![AmiType::AL2023_x86_64_NVIDIA, AmiType::BOTTLEROCKET_x86_64_NVIDIA])]
  #[case(AcceleratorType::Nvidia, true, CpuArch::Arm64, vec![AmiType::AL2023_ARM_64_NVIDIA, AmiType::BOTTLEROCKET_ARM_64_NVIDIA])]
  #[case(AcceleratorType::Neuron, false, CpuArch::X8664, vec![AmiType::AL2023_x86_64_NEURON, AmiType::BOTTLEROCKET_x86_64])]
  #[case(AcceleratorType::Neuron, false, CpuArch::Arm64, vec![AmiType::BOTTLEROCKET_ARM_64])]
  #[case(AcceleratorType::Neuron, true, CpuArch::X8664, vec![AmiType::AL2023_x86_64_NEURON, AmiType::BOTTLEROCKET_x86_64])]
  #[case(AcceleratorType::Neuron, true, CpuArch::Arm64, vec![AmiType::BOTTLEROCKET_ARM_64])]
  #[case(AcceleratorType::None, false, CpuArch::X8664, vec![
    AmiType::AL2023_x86_64_STANDARD,
    AmiType::BOTTLEROCKET_x86_64,
    AmiType::WINDOWS_CORE_2019_x86_64,
    AmiType::WINDOWS_CORE_2022_x86_64,
    AmiType::WINDOWS_FULL_2019_x86_64,
    AmiType::WINDOWS_FULL_2022_x86_64,
  ])]
  #[case(AcceleratorType::None, false, CpuArch::Arm64, vec![AmiType::AL2023_ARM_64_STANDARD, AmiType::BOTTLEROCKET_ARM_64])]
  #[case(AcceleratorType::None, true, CpuArch::X8664, vec![AmiType::AL2023_x86_64_NVIDIA, AmiType::AL2023_x86_64_NEURON])]
  #[case(AcceleratorType::None, true, CpuArch::Arm64, vec![AmiType::AL2023_ARM_64_NVIDIA, AmiType::BOTTLEROCKET_ARM_64])]
  fn test_get_ami_types(
    #[case] accelerator: AcceleratorType,
    #[case] require_efa: bool,
    #[case] cpu_arch: CpuArch,
    #[case] expected: Vec<AmiType>,
  ) {
    let ami_types = get_ami_types(&accelerator, require_efa, &cpu_arch);
    assert_eq!(ami_types, expected);
  }

  #[rstest]
  #[case(AmiType::AL2023_x86_64_NEURON, CpuArch::X8664, AmiType::AL2023_x86_64_STANDARD)]
  #[case(AmiType::AL2023_x86_64_NVIDIA, CpuArch::X8664, AmiType::AL2023_x86_64_STANDARD)]
  #[case(AmiType::AL2023_ARM_64_STANDARD, CpuArch::Arm64, AmiType::AL2023_ARM_64_STANDARD)]
  #[case(AmiType::AL2023_x86_64_STANDARD, CpuArch::X8664, AmiType::AL2023_x86_64_STANDARD)]
  #[case(AmiType::BOTTLEROCKET_x86_64_NVIDIA, CpuArch::X8664, AmiType::BOTTLEROCKET_x86_64)]
  #[case(AmiType::BOTTLEROCKET_ARM_64_NVIDIA, CpuArch::Arm64, AmiType::BOTTLEROCKET_ARM_64)]
  #[case(AmiType::BOTTLEROCKET_ARM_64, CpuArch::Arm64, AmiType::BOTTLEROCKET_ARM_64)]
  #[case(AmiType::WINDOWS_CORE_2019_x86_64, CpuArch::X8664, AmiType::AL2023_x86_64_STANDARD)]
  fn test_get_default_ami_type(#[case] ami_type: AmiType, #[case] cpu_arch: CpuArch, #[case] expected: AmiType) {
    let default_ami_type = get_default_ami_type(&ami_type, &cpu_arch);
    assert_eq!(default_ami_type, expected);
  }
}
