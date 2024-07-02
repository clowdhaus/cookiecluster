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
  #[serde(rename = "AL2_ARM_64")]
  Al2Arm64,
  #[serde(rename = "AL2_x86_64")]
  Al2X8664,
  #[serde(rename = "AL2_x86_64_GPU")]
  Al2X8664Gpu,
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
      AcceleratorType::Nvidia => {
        if enable_efa {
          vec!["AL2_x86_64_GPU", "CUSTOM"]
        } else {
          match cpu_arch {
            CpuArch::Arm64 => vec!["BOTTLEROCKET_ARM_64_NVIDIA", "CUSTOM"],
            _ => vec!["AL2_x86_64_GPU", "BOTTLEROCKET_x86_64_NVIDIA", "CUSTOM"],
          }
        }
      }
      AcceleratorType::Neuron => {
        vec!["AL2_x86_64_GPU", "CUSTOM"]
      }
      _ => match cpu_arch {
        CpuArch::Arm64 => {
          vec![
            "AL2023_ARM_64_STANDARD",
            "AL2_ARM_64",
            "BOTTLEROCKET_ARM_64",
            "BOTTLEROCKET_ARM_64_NVIDIA",
            "CUSTOM",
          ]
        }
        _ => {
          vec![
            "AL2023_x86_64_STANDARD",
            "AL2_x86_64",
            "BOTTLEROCKET_x86_64",
            "BOTTLEROCKET_x86_64_NVIDIA",
            "WINDOWS_CORE_2019_x86_64",
            "WINDOWS_CORE_2022_x86_64",
            "WINDOWS_FULL_2019_x86_64",
            "WINDOWS_FULL_2022_x86_64",
            "CUSTOM",
          ]
        }
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
      AmiType::Al2Arm64 => write!(f, "AL2_ARM_64"),
      AmiType::Al2X8664 => write!(f, "AL2_x86_64"),
      AmiType::Al2X8664Gpu => write!(f, "AL2_x86_64_GPU"),
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
      "AL2_ARM_64" => AmiType::Al2Arm64,
      "AL2_x86_64" => AmiType::Al2X8664,
      "AL2_x86_64_GPU" => AmiType::Al2X8664Gpu,
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

/// Get the default AMI type equivalent based on the accelerator type, AMI type, and CPU architecture
pub fn get_default_ami_type(accelerator: &AcceleratorType, ami_type: &AmiType, cpu_arch: &CpuArch) -> AmiType {
  match accelerator {
    AcceleratorType::Nvidia | AcceleratorType::Neuron => match ami_type {
      AmiType::Al2X8664Gpu => AmiType::Al2023X8664Standard,
      AmiType::BottlerocketX8664Nvidia | AmiType::BottlerocketArm64Nvidia => AmiType::BottlerocketX8664,
      _ => AmiType::Al2023X8664Standard,
    },
    _ => match cpu_arch {
      CpuArch::X8664 => AmiType::Al2023X8664Standard,
      CpuArch::Arm64 => AmiType::Al2023X8664Standard,
    },
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_ami_type_from_str() {
    let ami_type = AmiType::from("AL2_x86_64");
    assert_eq!(ami_type, AmiType::Al2X8664);

    let ami_type = AmiType::from("DOES_NOT_EXIST");
    assert_eq!(ami_type, AmiType::Custom);
  }
}
