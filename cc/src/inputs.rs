use std::fmt;

use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};

use crate::INSTANCE_TYPES;

pub const REMOVED_INSTANCE_TYPES: &[&str] = &[""];

#[derive(Debug)]
pub struct Inputs {
  enable_efa: bool,
  accelerator: AcceleratorType,
  reservation: ReservationType,
  compute_scaling: ComputeScalingType,
  cpu_arch: CpuArch,
  instances: Vec<String>,
  ami_type: AmiTypes,
}

impl Default for Inputs {
  fn default() -> Self {
    Inputs {
      enable_efa: false,
      accelerator: AcceleratorType::None,
      reservation: ReservationType::None,
      compute_scaling: ComputeScalingType::None,
      cpu_arch: CpuArch::X8664,
      instances: vec![],
      ami_type: AmiTypes::Al2023X8664Standard,
    }
  }
}

#[derive(Debug, PartialEq)]
enum AcceleratorType {
  Nvidia,
  Neuron,
  None,
}

#[derive(Debug, PartialEq)]
enum ReservationType {
  OnDemandCapacityReservation,
  MlCapacityBlockReservation,
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

#[derive(Debug, PartialEq)]
enum ComputeScalingType {
  ClusterAutoscaler,
  Karpenter,
  None,
}

#[derive(Debug, PartialEq)]
enum CpuArch {
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

impl Inputs {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn collect(self) -> Result<Self> {
    let inputs = self
      .collect_accelerator_type()?
      .collect_enable_efa()?
      .collect_reservation_type()?
      .collect_compute_scaling_type()?
      .collect_cpu_arch()?
      .collect_ami_type()?
      .collect_instance_types()?;

    Ok(inputs)
  }

  fn collect_enable_efa(mut self) -> Result<Inputs> {
    match self.accelerator {
      AcceleratorType::Nvidia | AcceleratorType::Neuron => {
        self.enable_efa = Confirm::with_theme(&ColorfulTheme::default())
          .with_prompt("Enable EFA support")
          .interact()?
      }
      _ => {}
    }

    Ok(self)
  }

  fn collect_accelerator_type(mut self) -> Result<Inputs> {
    let accelerator_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Accelerator type")
      .item("None")
      .item("NVIDIA GPU")
      .item("AWS Neuron")
      .default(0)
      .interact()?;

    let accelerator = match accelerator_idx {
      1 => AcceleratorType::Nvidia,
      2 => AcceleratorType::Neuron,
      _ => AcceleratorType::None,
    };
    self.accelerator = accelerator;

    Ok(self)
  }

  fn collect_reservation_type(mut self) -> Result<Inputs> {
    let items = match self.accelerator {
      AcceleratorType::Nvidia => vec![
        "None",
        "On-demand capacity reservation",
        "ML capacity block reservation",
      ],
      _ => vec!["None", "On-demand capacity reservation"],
    };

    let reservation_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("EC2 capacity reservation")
      .items(&items[..])
      .default(0)
      .interact()?;

    self.reservation = ReservationType::from(items[reservation_idx]);
    Ok(self)
  }

  fn collect_compute_scaling_type(mut self) -> Result<Inputs> {
    let mut items = vec!["None", "cluster-autoscaler"];

    if self.reservation == ReservationType::None {
      items.push("Karpenter");
    }

    let compute_scaling_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Compute autoscaling")
      .items(&items[..])
      .default(0)
      .interact()?;

    let compute_scaling = match compute_scaling_idx {
      1 => ComputeScalingType::ClusterAutoscaler,
      2 => ComputeScalingType::Karpenter,
      _ => ComputeScalingType::None,
    };
    self.compute_scaling = compute_scaling;

    Ok(self)
  }

  fn collect_cpu_arch(mut self) -> Result<Inputs> {
    // Set on Karpenter NodeClass
    if self.compute_scaling == ComputeScalingType::Karpenter {
      return Ok(self);
    }

    // Inf/Trn instances only support x86-64 at this time
    if self.accelerator == AcceleratorType::Neuron {
      return Ok(self);
    }

    if self.accelerator == AcceleratorType::Nvidia && self.enable_efa {
      return Ok(self);
    }

    let cpu_arch_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("CPU architecture")
      .item("x86-64")
      .item("arm64")
      .default(0)
      .interact()?;

    let cpu_arch = match cpu_arch_idx {
      1 => CpuArch::Arm64,
      _ => CpuArch::X8664,
    };
    self.cpu_arch = cpu_arch;

    Ok(self)
  }

  fn collect_ami_type(mut self) -> Result<Inputs> {
    // Set on Karpenter NodeClass
    if self.compute_scaling == ComputeScalingType::Karpenter {
      return Ok(self);
    }

    let ami_types = match self.accelerator {
      AcceleratorType::Nvidia => {
        if self.enable_efa {
          vec!["AL2_x86_64_GPU", "CUSTOM"]
        } else {
          match self.cpu_arch {
            CpuArch::Arm64 => vec!["BOTTLEROCKET_ARM_64_NVIDIA", "CUSTOM"],
            _ => vec!["AL2_x86_64_GPU", "BOTTLEROCKET_x86_64_NVIDIA", "CUSTOM"],
          }
        }
      }
      AcceleratorType::Neuron => {
        vec!["AL2_x86_64", "CUSTOM"]
      }
      _ => match self.cpu_arch {
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

    let ami_type_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("AMI type")
      .items(&ami_types[..])
      .interact()?;

    self.ami_type = AmiTypes::from(ami_types[ami_type_idx]);

    Ok(self)
  }

  fn collect_instance_types(mut self) -> Result<Inputs> {
    // Set on Karpenter NodeClass
    if self.compute_scaling == ComputeScalingType::Karpenter {
      return Ok(self);
    }

    let instance_types = INSTANCE_TYPES
      .iter()
      .filter(|i| {
        i.cpu_arch == self.cpu_arch.to_string()
          && if self.enable_efa { i.efa_supported } else { true }
          && if self.accelerator == AcceleratorType::Nvidia {
            i.nvidia_gpu_supported
          } else if self.accelerator == AcceleratorType::Neuron {
            i.neuron_supported
          } else {
            true
          }
          && if self.reservation == ReservationType::MlCapacityBlockReservation {
            i.cbr_supported
          } else {
            true
          }
      })
      .map(|i| i.instance_type.to_string())
      .collect::<Vec<String>>();

    let instance_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Instance type(s)")
      .items(&instance_types)
      .interact()?;

    let instances = instance_idxs
      .iter()
      .map(|&i| instance_types[i].to_string())
      .collect::<Vec<String>>();

    if instances.is_empty() {
      bail!("At least one instance type needs to be selected");
    }

    self.instances = instances;

    Ok(self)
  }
}

#[derive(Debug, PartialEq)]
enum AmiTypes {
  Al2023Arm64Standard,
  Al2023X8664Standard,
  Al2Arm64,
  Al2X8664,
  Al2X8664Gpu,
  BottlerocketArm64,
  BottlerocketArm64Nvidia,
  BottlerocketX8664,
  BottlerocketX8664Nvidia,
  Custom,
  WindowsCore2019X8664,
  WindowsCore2022X8664,
  WindowsFull2019X8664,
  WindowsFull2022X8664,
}

impl std::fmt::Display for AmiTypes {
  fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    match self {
      AmiTypes::Al2023Arm64Standard => write!(f, "AL2023_ARM_64_STANDARD"),
      AmiTypes::Al2023X8664Standard => write!(f, "AL2023_x86_64_STANDARD"),
      AmiTypes::Al2Arm64 => write!(f, "AL2_ARM_64"),
      AmiTypes::Al2X8664 => write!(f, "AL2_x86_64"),
      AmiTypes::Al2X8664Gpu => write!(f, "AL2_x86_64_GPU"),
      AmiTypes::BottlerocketArm64 => write!(f, "BOTTLEROCKET_ARM_64"),
      AmiTypes::BottlerocketArm64Nvidia => write!(f, "BOTTLEROCKET_ARM_64_NVIDIA"),
      AmiTypes::BottlerocketX8664 => write!(f, "BOTTLEROCKET_x86_64"),
      AmiTypes::BottlerocketX8664Nvidia => write!(f, "BOTTLEROCKET_x86_64_NVIDIA"),
      AmiTypes::Custom => write!(f, "CUSTOM"),
      AmiTypes::WindowsCore2019X8664 => write!(f, "WINDOWS_CORE_2019_x86_64"),
      AmiTypes::WindowsCore2022X8664 => write!(f, "WINDOWS_CORE_2022_x86_64"),
      AmiTypes::WindowsFull2019X8664 => write!(f, "WINDOWS_FULL_2019_x86_64"),
      AmiTypes::WindowsFull2022X8664 => write!(f, "WINDOWS_FULL_2022_x86_64"),
    }
  }
}

impl std::convert::From<&str> for AmiTypes {
  fn from(s: &str) -> Self {
    match s {
      "AL2023_ARM_64_STANDARD" => AmiTypes::Al2023Arm64Standard,
      "AL2023_x86_64_STANDARD" => AmiTypes::Al2023X8664Standard,
      "AL2_ARM_64" => AmiTypes::Al2Arm64,
      "AL2_x86_64" => AmiTypes::Al2X8664,
      "AL2_x86_64_GPU" => AmiTypes::Al2X8664Gpu,
      "BOTTLEROCKET_ARM_64" => AmiTypes::BottlerocketArm64,
      "BOTTLEROCKET_ARM_64_NVIDIA" => AmiTypes::BottlerocketArm64Nvidia,
      "BOTTLEROCKET_x86_64" => AmiTypes::BottlerocketX8664,
      "BOTTLEROCKET_x86_64_NVIDIA" => AmiTypes::BottlerocketX8664Nvidia,
      "WINDOWS_CORE_2019_x86_64" => AmiTypes::WindowsCore2019X8664,
      "WINDOWS_CORE_2022_x86_64" => AmiTypes::WindowsCore2022X8664,
      "WINDOWS_FULL_2019_x86_64" => AmiTypes::WindowsFull2019X8664,
      "WINDOWS_FULL_2022_x86_64" => AmiTypes::WindowsFull2022X8664,
      _ => AmiTypes::Custom,
    }
  }
}
