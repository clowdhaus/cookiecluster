mod add_on;
mod ami;
mod compute;
mod instances;
mod version;

use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[derive(Debug, Serialize, Deserialize)]
pub struct Inputs {
  accelerator: compute::AcceleratorType,
  pub add_ons: Vec<add_on::AddOn>,
  ami_type: ami::AmiType,
  cluster_endpoint_public_access: bool,
  cluster_name: String,
  cluster_version: version::ClusterVersion,
  control_plane_subnet_filter: String,
  pub compute_scaling: compute::ScalingType,
  cpu_arch: compute::CpuArch,
  data_plane_subnet_filter: String,
  /// AMI type used on default node group when secondary node group (accelerated, Windows, etc) is used
  default_ami_type: ami::AmiType,
  /// Instance types used on default node group when secondary node group (accelerated, Windows, etc) is used
  default_instance_types: Vec<String>,
  enable_cluster_creator_admin_permissions: bool,
  enable_efa: bool,
  instance_storage_supported: bool,
  instance_types: Vec<String>,
  reservation: compute::ReservationType,
  reservation_availability_zone: String,
  vpc_name: String,
}

impl Default for Inputs {
  fn default() -> Self {
    Inputs {
      accelerator: compute::AcceleratorType::None,
      add_ons: vec![],
      ami_type: ami::AmiType::Al2023X8664Standard,
      cluster_endpoint_public_access: false,
      cluster_name: String::from("example"),
      cluster_version: version::ClusterVersion::K130,
      control_plane_subnet_filter: String::from("*-private-*"),
      compute_scaling: compute::ScalingType::None,
      cpu_arch: compute::CpuArch::X8664,
      data_plane_subnet_filter: String::from("*-private-*"),
      default_ami_type: ami::AmiType::Al2023X8664Standard,
      default_instance_types: vec![],
      enable_cluster_creator_admin_permissions: false,
      enable_efa: false,
      instance_storage_supported: false,
      instance_types: vec![],
      reservation: compute::ReservationType::None,
      reservation_availability_zone: String::from("us-west-2a"),
      vpc_name: String::from("example"),
    }
  }
}

impl Inputs {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn collect(self) -> Result<Self> {
    let inputs = self
      .collect_cluster_settings()?
      .collect_add_ons()?
      .collect_accelerator_type()?
      .collect_enable_efa()?
      .collect_reservation_type()?
      .collect_compute_scaling_type()?
      .collect_networking_settings()?
      .collect_cpu_arch()?
      .collect_ami_type()?
      .collect_instance_types()?
      .collect_storage_settings()?
      .collect_default_node_group_settings()?;

    Ok(inputs)
  }

  fn collect_cluster_settings(mut self) -> Result<Inputs> {
    self.cluster_name = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Cluster name")
      .interact_text()?;

    // This is ugly
    // TODO - find better way to get from enum variants to &[&str]
    let cluster_versions = version::ClusterVersion::iter()
      .map(|v| v.to_string())
      .collect::<Vec<_>>();
    let cluster_versions: Vec<&str> = cluster_versions.iter().map(|s| s as &str).collect();

    let cluster_version_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Cluster version")
      .items(&cluster_versions[..])
      .default(0)
      .interact()?;
    self.cluster_version = version::ClusterVersion::from(cluster_versions[cluster_version_idx]);

    self.cluster_endpoint_public_access = Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt("Enable public access to cluster endpoint")
      .default(false)
      .interact()?;

    self.enable_cluster_creator_admin_permissions = Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt("Enable admin permissions for cluster creator")
      .default(false)
      .interact()?;

    Ok(self)
  }

  fn collect_add_ons(mut self) -> Result<Inputs> {
    // This is ugly
    // TODO - find better way to get from enum variants to &[&str]
    let all_add_ons = add_on::AddOnType::iter().map(|v| v.to_string()).collect::<Vec<_>>();
    let all_add_ons: Vec<&str> = all_add_ons.iter().map(|s| s as &str).collect();

    let add_ons_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("EKS add-on(s)")
      .items(&all_add_ons[..])
      .defaults(&[true, true, true, true])
      .interact()?;

    let add_ons = add_ons_idxs
      .iter()
      .map(|&i| {
        let add_on = add_on::AddOnType::from(all_add_ons[i]);
        match add_on {
          // Not adding vpc-cni since it still requires permissions on node IAM role to start
          add_on::AddOnType::AwsEbsCsiDriver
          | add_on::AddOnType::AwsEfsCsiDriver
          | add_on::AddOnType::AwsMountpointS3CsiDriver
          | add_on::AddOnType::AmazonCloudwatchObservability => {
            let under_name = add_on.to_string().replace('-', "_");
            add_on::AddOn {
              name: add_on.to_string(),
              under_name: under_name.to_string(),
              configuration: add_on::AddOnConfiguration {
                service_account_role_arn: Some(format!("module.{under_name}_irsa.iam_role_arn")),
              },
            }
          }
          _ => add_on::AddOn {
            name: add_on.to_string(),
            under_name: add_on.to_string().replace('-', "_"),
            configuration: add_on::AddOnConfiguration {
              service_account_role_arn: None,
            },
          },
        }
      })
      .collect::<Vec<add_on::AddOn>>();

    self.add_ons = add_ons;

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
      1 => compute::AcceleratorType::Nvidia,
      2 => compute::AcceleratorType::Neuron,
      _ => compute::AcceleratorType::None,
    };
    self.accelerator = accelerator;

    Ok(self)
  }

  fn collect_enable_efa(mut self) -> Result<Inputs> {
    match self.accelerator {
      compute::AcceleratorType::Nvidia | compute::AcceleratorType::Neuron => {
        self.enable_efa = Confirm::with_theme(&ColorfulTheme::default())
          .with_prompt("Enable EFA support")
          .default(true)
          .interact()?
      }
      _ => {}
    }

    Ok(self)
  }

  fn collect_reservation_type(mut self) -> Result<Inputs> {
    let items = match self.accelerator {
      compute::AcceleratorType::Nvidia => vec![
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

    self.reservation = compute::ReservationType::from(items[reservation_idx]);
    Ok(self)
  }

  fn collect_compute_scaling_type(mut self) -> Result<Inputs> {
    let mut items = vec!["cluster-autoscaler", "None"];
    if self.reservation == compute::ReservationType::None {
      items = vec!["karpenter", "cluster-autoscaler", "None"];
    }

    let compute_scaling_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Compute autoscaling")
      .items(&items[..])
      .default(0)
      .interact()?;

    self.compute_scaling = compute::ScalingType::from(items[compute_scaling_idx]);

    Ok(self)
  }

  fn collect_networking_settings(mut self) -> Result<Inputs> {
    self.vpc_name = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("VPC name")
      .with_initial_text("example".to_string())
      .interact_text()?;

    self.control_plane_subnet_filter = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Control plane subnet filter")
      .with_initial_text("*-private-*".to_string())
      .interact_text()?;

    self.data_plane_subnet_filter = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Data plane subnet filter")
      .with_initial_text("*-private-*".to_string())
      .interact_text()?;

    if self.reservation != compute::ReservationType::None {
      self.reservation_availability_zone = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("EC2 capacity reservation availability zone")
        .with_initial_text("us-west-2a".to_string())
        .interact_text()?;
    }

    Ok(self)
  }

  fn collect_cpu_arch(mut self) -> Result<Inputs> {
    // Set on Karpenter NodeClass
    if self.compute_scaling == compute::ScalingType::Karpenter {
      return Ok(self);
    }

    // Inf/Trn instances only support x86-64 at this time
    if self.accelerator == compute::AcceleratorType::Neuron {
      return Ok(self);
    }

    if self.accelerator == compute::AcceleratorType::Nvidia && self.enable_efa {
      return Ok(self);
    }

    let cpu_arch_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("CPU architecture")
      .item("x86-64")
      .item("arm64")
      .default(0)
      .interact()?;

    let cpu_arch = match cpu_arch_idx {
      1 => compute::CpuArch::Arm64,
      _ => compute::CpuArch::X8664,
    };
    self.cpu_arch = cpu_arch;

    Ok(self)
  }

  fn collect_ami_type(mut self) -> Result<Inputs> {
    let ami_types = ami::AmiType::get_ami_types(&self.accelerator, self.enable_efa, &self.cpu_arch)?;
    let ami_type_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("AMI type")
      .items(&ami_types[..])
      .default(0)
      .interact()?;

    self.ami_type = ami::AmiType::from(ami_types[ami_type_idx]);

    Ok(self)
  }

  fn collect_instance_types(mut self) -> Result<Inputs> {
    let instance_types = instances::INSTANCE_TYPES
      .iter()
      .filter(|i| {
        i.cpu_arch == self.cpu_arch.to_string()
          && if self.enable_efa { i.efa_supported } else { true }
          && if self.accelerator == compute::AcceleratorType::Nvidia {
            i.nvidia_gpu_supported
          } else if self.accelerator == compute::AcceleratorType::Neuron {
            i.neuron_supported
          } else {
            true
          }
          && if self.reservation == compute::ReservationType::MlCapacityBlockReservation {
            i.cbr_supported
          } else {
            true
          }
      })
      .map(|i| i.instance_type.to_string())
      .collect::<Vec<String>>();

    let mut instance_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Instance type(s)")
      .items(&instance_types)
      .interact()?;

    if instance_idxs.is_empty() {
      instance_idxs.push(0);
    }

    // There are two scenarios where only a single instance type should be specified:
    // 1. EC2 capacity reservation(s)
    // 2. When using EFA
    if self.reservation != compute::ReservationType::None || self.enable_efa {
      instance_idxs = vec![instance_idxs.last().unwrap().to_owned()];
    }

    let instance_types = instance_idxs
      .iter()
      .map(|&i| instance_types[i].to_string())
      .collect::<Vec<String>>();

    if instance_types.is_empty() {
      bail!("At least one instance type needs to be selected");
    }

    self.instance_types = instance_types;

    Ok(self)
  }

  fn collect_storage_settings(mut self) -> Result<Inputs> {
    let instance_types_support = instances::INSTANCE_TYPES
      .iter()
      .filter(|instance| self.instance_types.contains(&instance.instance_type.to_string()))
      .map(|instance| instance.instance_storage_supported)
      .all(|f| f);

    let instance_storage_supported = match self.ami_type {
      ami::AmiType::Al2023Arm64Standard
      | ami::AmiType::Al2023X8664Standard
      | ami::AmiType::Al2Arm64
      | ami::AmiType::Al2X8664
      | ami::AmiType::Al2X8664Gpu => instance_types_support,
      _ => false,
    };
    self.instance_storage_supported = instance_storage_supported;

    Ok(self)
  }

  fn collect_default_node_group_settings(mut self) -> Result<Inputs> {
    // Based on the AMI type selected, set the default AMI type equivalent for the default node group
    self.default_ami_type = match self.accelerator {
      compute::AcceleratorType::Nvidia | compute::AcceleratorType::Neuron => match self.ami_type {
        ami::AmiType::Al2X8664Gpu => ami::AmiType::Al2023X8664Standard,
        ami::AmiType::BottlerocketX8664Nvidia | ami::AmiType::BottlerocketArm64Nvidia => {
          ami::AmiType::BottlerocketX8664
        }
        _ => ami::AmiType::Al2023X8664Standard,
      },
      _ => match self.cpu_arch {
        compute::CpuArch::X8664 => ami::AmiType::Al2023X8664Standard,
        compute::CpuArch::Arm64 => ami::AmiType::Al2023X8664Standard,
      },
    };

    // Based on the default AMI type selected, set the default instance type(s) for the default node group
    self.default_instance_types = match self.default_ami_type {
      ami::AmiType::Al2Arm64 | ami::AmiType::Al2X8664 | ami::AmiType::BottlerocketArm64Nvidia => {
        vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()]
      }
      _ => vec!["m7a.xlarge".to_string(), "m7i.xlarge".to_string()],
    };

    Ok(self)
  }
}
