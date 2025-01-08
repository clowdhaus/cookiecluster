use std::fmt;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::compute::{AcceleratorType, CpuArch};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum AmiType {
  #[serde(rename = "AL2023_ARM_64_STANDARD")]
  Al2023Arm64Standard,
  #[serde(rename = "AL2023_x86_64_STANDARD")]
  Al2023X8664Standard,
  #[serde(rename = "AL2023_x86_64_NEURON")]
  Al2023X8664Neuron,
  #[serde(rename = "AL2023_x86_64_NVIDIA")]
  Al2023X8664Nvidia,
  #[serde(rename = "BOTTLEROCKET_ARM_64")]
  BottlerocketArm64,
  #[serde(rename = "BOTTLEROCKET_ARM_64_NVIDIA")]
  BottlerocketArm64Nvidia,
  #[serde(rename = "BOTTLEROCKET_x86_64")]
  BottlerocketX8664,
  #[serde(rename = "BOTTLEROCKET_x86_64_NVIDIA")]
  BottlerocketX8664Nvidia,
  #[serde(rename = "CUSTOM")]
  Custom,
  #[serde(rename = "WINDOWS_CORE_2019_x86_64")]
  WindowsCore2019X8664,
  #[serde(rename = "WINDOWS_CORE_2022_x86_64")]
  WindowsCore2022X8664,
  #[serde(rename = "WINDOWS_FULL_2019_x86_64")]
  WindowsFull2019X8664,
  #[serde(rename = "WINDOWS_FULL_2022_x86_64")]
  WindowsFull2022X8664,
}

impl AmiType {
  pub fn get_ami_types<'a>(
    accelerator: &'a AcceleratorType,
    enable_efa: bool,
    cpu_arch: &'a CpuArch,
  ) -> Result<Vec<&'a str>> {
    let ami_types = match accelerator {
      AcceleratorType::Nvidia => match (cpu_arch, enable_efa) {
        (CpuArch::X8664, true) => vec!["AL2023_x86_64_NVIDIA"],
        (CpuArch::X8664, false) => vec!["AL2023_x86_64_NVIDIA", "BOTTLEROCKET_x86_64_NVIDIA"],
        (CpuArch::Arm64, true) => vec![],
        (CpuArch::Arm64, false) => vec!["BOTTLEROCKET_ARM_64_NVIDIA"],
      },
      AcceleratorType::Neuron => match (cpu_arch, enable_efa) {
        (CpuArch::X8664, true) => vec!["AL2023_x86_64_NEURON"],
        (CpuArch::X8664, false) => vec!["AL2023_x86_64_NEURON"],
        (CpuArch::Arm64, true) => vec![],
        (CpuArch::Arm64, false) => vec![],
      },
      _ => match (cpu_arch, enable_efa) {
        (CpuArch::X8664, true) => vec!["AL2023_x86_64_NVIDIA", "AL2023_x86_64_NEURON"],
        (CpuArch::X8664, false) => vec![
          "AL2023_x86_64_STANDARD",
          "BOTTLEROCKET_x86_64",
          "WINDOWS_CORE_2019_x86_64",
          "WINDOWS_CORE_2022_x86_64",
          "WINDOWS_FULL_2019_x86_64",
          "WINDOWS_FULL_2022_x86_64",
        ],
        (CpuArch::Arm64, true) => vec![],
        (CpuArch::Arm64, false) => vec!["AL2023_ARM_64_STANDARD", "BOTTLEROCKET_ARM_64"],
      },
    };

    Ok(ami_types)
  }
}

impl std::fmt::Display for AmiType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AmiType::Al2023Arm64Standard => write!(f, "AL2023_ARM_64_STANDARD"),
      AmiType::Al2023X8664Standard => write!(f, "AL2023_x86_64_STANDARD"),
      AmiType::Al2023X8664Neuron => write!(f, "AL2023_x86_64_NEURON"),
      AmiType::Al2023X8664Nvidia => write!(f, "AL2023_x86_64_NVIDIA"),
      AmiType::BottlerocketArm64 => write!(f, "BOTTLEROCKET_ARM_64"),
      AmiType::BottlerocketArm64Nvidia => write!(f, "BOTTLEROCKET_ARM_64_NVIDIA"),
      AmiType::BottlerocketX8664 => write!(f, "BOTTLEROCKET_x86_64"),
      AmiType::BottlerocketX8664Nvidia => write!(f, "BOTTLEROCKET_x86_64_NVIDIA"),
      AmiType::Custom => write!(f, "CUSTOM"),
      AmiType::WindowsCore2019X8664 => write!(f, "WINDOWS_CORE_2019_x86_64"),
      AmiType::WindowsCore2022X8664 => write!(f, "WINDOWS_CORE_2022_x86_64"),
      AmiType::WindowsFull2019X8664 => write!(f, "WINDOWS_FULL_2019_x86_64"),
      AmiType::WindowsFull2022X8664 => write!(f, "WINDOWS_FULL_2022_x86_64"),
    }
  }
}

impl std::convert::From<&str> for AmiType {
  fn from(s: &str) -> Self {
    match s {
      "AL2023_ARM_64_STANDARD" => AmiType::Al2023Arm64Standard,
      "AL2023_x86_64_STANDARD" => AmiType::Al2023X8664Standard,
      "AL2023_x86_64_NEURON" => AmiType::Al2023X8664Neuron,
      "AL2023_x86_64_NVIDIA" => AmiType::Al2023X8664Nvidia,
      "BOTTLEROCKET_ARM_64" => AmiType::BottlerocketArm64,
      "BOTTLEROCKET_ARM_64_NVIDIA" => AmiType::BottlerocketArm64Nvidia,
      "BOTTLEROCKET_x86_64" => AmiType::BottlerocketX8664,
      "BOTTLEROCKET_x86_64_NVIDIA" => AmiType::BottlerocketX8664Nvidia,
      "WINDOWS_CORE_2019_x86_64" => AmiType::WindowsCore2019X8664,
      "WINDOWS_CORE_2022_x86_64" => AmiType::WindowsCore2022X8664,
      "WINDOWS_FULL_2019_x86_64" => AmiType::WindowsFull2019X8664,
      "WINDOWS_FULL_2022_x86_64" => AmiType::WindowsFull2022X8664,
      _ => AmiType::Custom,
    }
  }
}

/// Get the default AMI type equivalent based on the AMI type, and CPU architecture
pub fn get_default_ami_type(ami_type: &AmiType, cpu_arch: &CpuArch) -> AmiType {
  match ami_type {
    AmiType::BottlerocketX8664Nvidia | AmiType::BottlerocketX8664 => AmiType::BottlerocketX8664,
    AmiType::BottlerocketArm64Nvidia | AmiType::BottlerocketArm64 => AmiType::BottlerocketArm64,
    _ => match cpu_arch {
      CpuArch::X8664 => AmiType::Al2023X8664Standard,
      CpuArch::Arm64 => AmiType::Al2023Arm64Standard,
    },
  }
}

#[cfg(test)]
mod tests {

  use rstest::*;

  use super::*;

  #[rstest]
  #[case(AcceleratorType::Nvidia, false, CpuArch::X8664, vec!["AL2023_x86_64_NVIDIA", "BOTTLEROCKET_x86_64_NVIDIA"])]
  #[case(AcceleratorType::Nvidia, false, CpuArch::Arm64, vec!["BOTTLEROCKET_ARM_64_NVIDIA"])]
  #[case(AcceleratorType::Nvidia, true, CpuArch::X8664, vec!["AL2023_x86_64_NVIDIA"])]
  #[case(AcceleratorType::Nvidia, true, CpuArch::Arm64, vec![])]
  #[case(AcceleratorType::Neuron, false, CpuArch::X8664, vec!["AL2023_x86_64_NEURON"])]
  #[case(AcceleratorType::Neuron, false, CpuArch::Arm64, vec![])]
  #[case(AcceleratorType::Neuron, true, CpuArch::X8664, vec!["AL2023_x86_64_NEURON"])]
  #[case(AcceleratorType::Neuron, true, CpuArch::Arm64, vec![])]
  #[case(AcceleratorType::None, false, CpuArch::X8664, vec![
    "AL2023_x86_64_STANDARD",
    "BOTTLEROCKET_x86_64",
    "WINDOWS_CORE_2019_x86_64",
    "WINDOWS_CORE_2022_x86_64",
    "WINDOWS_FULL_2019_x86_64",
    "WINDOWS_FULL_2022_x86_64",
  ])]
  #[case(AcceleratorType::None, false, CpuArch::Arm64, vec!["AL2023_ARM_64_STANDARD", "BOTTLEROCKET_ARM_64"])]
  #[case(AcceleratorType::None, true, CpuArch::X8664, vec!["AL2023_x86_64_NVIDIA", "AL2023_x86_64_NEURON"])]
  #[case(AcceleratorType::None, true, CpuArch::Arm64, vec![])]
  fn test_get_ami_types(
    #[case] accelerator: AcceleratorType,
    #[case] enable_efa: bool,
    #[case] cpu_arch: CpuArch,
    #[case] expected: Vec<&str>,
  ) {
    let ami_types = AmiType::get_ami_types(&accelerator, enable_efa, &cpu_arch).unwrap();
    assert_eq!(ami_types, expected);
  }

  #[rstest]
  #[case(AmiType::Al2023X8664Neuron, CpuArch::X8664, AmiType::Al2023X8664Standard)]
  #[case(AmiType::Al2023X8664Nvidia, CpuArch::X8664, AmiType::Al2023X8664Standard)]
  #[case(AmiType::BottlerocketX8664Nvidia, CpuArch::X8664, AmiType::BottlerocketX8664)]
  #[case(AmiType::BottlerocketArm64Nvidia, CpuArch::Arm64, AmiType::BottlerocketArm64)]
  #[case(AmiType::WindowsCore2019X8664, CpuArch::X8664, AmiType::Al2023X8664Standard)]
  #[case(AmiType::BottlerocketArm64, CpuArch::Arm64, AmiType::BottlerocketArm64)]
  fn test_get_default_ami_type(#[case] ami_type: AmiType, #[case] cpu_arch: CpuArch, #[case] expected: AmiType) {
    let default_ami_type = get_default_ami_type(&ami_type, &cpu_arch);
    assert_eq!(default_ami_type, expected);
  }
}
