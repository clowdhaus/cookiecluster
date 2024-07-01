use std::fmt;

use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::INSTANCE_TYPES;

#[derive(Debug, Serialize, Deserialize)]
pub struct Inputs {
  accelerator: AcceleratorType,
  pub add_ons: Vec<AddOn>,
  ami_type: AmiType,
  cluster_endpoint_public_access: bool,
  cluster_name: String,
  cluster_version: ClusterVersion,
  control_plane_subnet_filter: String,
  pub compute_scaling: ComputeScalingType,
  cpu_arch: CpuArch,
  data_plane_subnet_filter: String,
  /// AMI type used on default node group when secondary node group (accelerated, Windows, etc) is used
  default_ami_type: AmiType,
  /// Instance types used on default node group when secondary node group (accelerated, Windows, etc) is used
  default_instance_types: Vec<String>,
  enable_cluster_creator_admin_permissions: bool,
  enable_efa: bool,
  instance_types: Vec<String>,
  pub reservation: ReservationType,
  reservation_availability_zone: String,
  vpc_name: String,
}

impl Default for Inputs {
  fn default() -> Self {
    Inputs {
      accelerator: AcceleratorType::None,
      add_ons: vec![],
      ami_type: AmiType::Al2023X8664Standard,
      cluster_endpoint_public_access: false,
      cluster_name: String::from("example"),
      cluster_version: ClusterVersion::K130,
      control_plane_subnet_filter: String::from("*-private-*"),
      compute_scaling: ComputeScalingType::None,
      cpu_arch: CpuArch::X8664,
      data_plane_subnet_filter: String::from("*-private-*"),
      default_ami_type: AmiType::Al2023X8664Standard,
      default_instance_types: vec![],
      enable_cluster_creator_admin_permissions: false,
      enable_efa: false,
      instance_types: vec![],
      reservation: ReservationType::None,
      reservation_availability_zone: String::from("us-west-2a"),
      vpc_name: String::from("example"),
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum AcceleratorType {
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
#[serde(rename_all = "kebab-case")]
enum AddOnType {
  CoreDns,
  KubeProxy,
  VpcCni,
  EksPodIdentityAgent,
  AwsEbsCsiDriver,
  AwsEfsCsiDriver,
  AwsMountpointS3CsiDriver,
  SnapshotController,
  Adot,
  AwsGuarddutyAgent,
  AmazonCloudwatchObservability,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AddOn {
  name: String,
  under_name: String,
  configuration: AddOnConfiguration,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AddOnConfiguration {
  service_account_role_arn: Option<String>,
}

impl std::fmt::Display for AddOnType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AddOnType::CoreDns => write!(f, "coredns"),
      AddOnType::KubeProxy => write!(f, "kube-proxy"),
      AddOnType::VpcCni => write!(f, "vpc-cni"),
      AddOnType::EksPodIdentityAgent => write!(f, "eks-pod-identity-agent"),
      AddOnType::AwsEbsCsiDriver => write!(f, "aws-ebs-csi-driver"),
      AddOnType::AwsEfsCsiDriver => write!(f, "aws-efs-csi-driver"),
      AddOnType::AwsMountpointS3CsiDriver => write!(f, "aws-mountpoint-s3-csi-driver"),
      AddOnType::SnapshotController => write!(f, "snapshot-controller"),
      AddOnType::Adot => write!(f, "adot"),
      AddOnType::AwsGuarddutyAgent => write!(f, "aws-guardduty-agent"),
      AddOnType::AmazonCloudwatchObservability => write!(f, "amazon-cloudwatch-observability"),
    }
  }
}

impl std::convert::From<&str> for AddOnType {
  fn from(s: &str) -> Self {
    match s {
      "coredns" => AddOnType::CoreDns,
      "kube-proxy" => AddOnType::KubeProxy,
      "vpc-cni" => AddOnType::VpcCni,
      "eks-pod-identity-agent" => AddOnType::EksPodIdentityAgent,
      "aws-ebs-csi-driver" => AddOnType::AwsEbsCsiDriver,
      "aws-efs-csi-driver" => AddOnType::AwsEfsCsiDriver,
      "aws-mountpoint-s3-csi-driver" => AddOnType::AwsMountpointS3CsiDriver,
      "snapshot-controller" => AddOnType::SnapshotController,
      "adot" => AddOnType::Adot,
      "aws-guardduty-agent" => AddOnType::AwsGuarddutyAgent,
      "amazon-cloudwatch-observability" => AddOnType::AmazonCloudwatchObservability,
      _ => AddOnType::CoreDns,
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum AmiType {
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

#[derive(Debug, EnumIter, PartialEq, Serialize, Deserialize)]
enum ClusterVersion {
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

#[derive(Debug, EnumIter, PartialEq, Serialize, Deserialize)]
pub enum ComputeScalingType {
  #[serde(rename = "karpenter")]
  Karpenter,
  #[serde(rename = "cluster-autoscaler")]
  ClusterAutoscaler,
  #[serde(rename = "None")]
  None,
}

impl std::convert::From<&str> for ComputeScalingType {
  fn from(s: &str) -> Self {
    match s {
      "karpenter" => ComputeScalingType::Karpenter,
      "cluster-autoscaler" => ComputeScalingType::ClusterAutoscaler,
      _ => ComputeScalingType::None,
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
      .collect_default_node_group_settings()?;

    Ok(inputs)
  }

  fn collect_cluster_settings(mut self) -> Result<Inputs> {
    self.cluster_name = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Cluster name")
      .interact_text()?;

    // This is ugly
    // TODO - find better way to get from enum variants to &[&str]
    let cluster_versions = ClusterVersion::iter().map(|v| v.to_string()).collect::<Vec<_>>();
    let cluster_versions: Vec<&str> = cluster_versions.iter().map(|s| s as &str).collect();

    let cluster_version_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Cluster version")
      .items(&cluster_versions[..])
      .default(0)
      .interact()?;
    self.cluster_version = ClusterVersion::from(cluster_versions[cluster_version_idx]);

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
    let all_add_ons = AddOnType::iter().map(|v| v.to_string()).collect::<Vec<_>>();
    let all_add_ons: Vec<&str> = all_add_ons.iter().map(|s| s as &str).collect();

    let add_ons_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("EKS add-on(s)")
      .items(&all_add_ons[..])
      .defaults(&[true, true, true, true])
      .interact()?;

    let add_ons = add_ons_idxs
      .iter()
      .map(|&i| {
        let add_on = AddOnType::from(all_add_ons[i]);
        match add_on {
          // Not adding vpc-cni since it still requires permissions on node IAM role to start
          AddOnType::AwsEbsCsiDriver
          | AddOnType::AwsEfsCsiDriver
          | AddOnType::AwsMountpointS3CsiDriver
          | AddOnType::AmazonCloudwatchObservability => {
            let under_name = add_on.to_string().replace('-', "_");
            AddOn {
              name: add_on.to_string(),
              under_name: under_name.to_string(),
              configuration: AddOnConfiguration {
                service_account_role_arn: Some(format!("module.{under_name}_irsa.iam_role_arn")),
              },
            }
          }
          _ => AddOn {
            name: add_on.to_string(),
            under_name: add_on.to_string().replace('-', "_"),
            configuration: AddOnConfiguration {
              service_account_role_arn: None,
            },
          },
        }
      })
      .collect::<Vec<AddOn>>();

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
      1 => AcceleratorType::Nvidia,
      2 => AcceleratorType::Neuron,
      _ => AcceleratorType::None,
    };
    self.accelerator = accelerator;

    Ok(self)
  }

  fn collect_enable_efa(mut self) -> Result<Inputs> {
    match self.accelerator {
      AcceleratorType::Nvidia | AcceleratorType::Neuron => {
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
    let mut items = vec!["cluster-autoscaler", "None"];
    if self.reservation == ReservationType::None {
      items = vec!["karpenter", "cluster-autoscaler", "None"];
    }

    let compute_scaling_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Compute autoscaling")
      .items(&items[..])
      .default(0)
      .interact()?;

    self.compute_scaling = ComputeScalingType::from(items[compute_scaling_idx]);

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

    if self.reservation != ReservationType::None {
      self.reservation_availability_zone = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("EC2 capacity reservation availability zone")
        .with_initial_text("us-west-2a".to_string())
        .interact_text()?;
    }

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
        vec!["AL2_x86_64_GPU", "CUSTOM"]
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
      .default(0)
      .interact()?;

    self.ami_type = AmiType::from(ami_types[ami_type_idx]);

    Ok(self)
  }

  fn collect_instance_types(mut self) -> Result<Inputs> {
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

    let mut instance_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Instance type(s)")
      .items(&instance_types)
      .interact()?;

    if instance_idxs.is_empty() {
      instance_idxs.push(0);
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

  fn collect_default_node_group_settings(mut self) -> Result<Inputs> {
    // Based on the AMI type selected, set the default AMI type equivalent for the default node group
    self.default_ami_type = match self.accelerator {
      AcceleratorType::Nvidia | AcceleratorType::Neuron => match self.ami_type {
        AmiType::Al2X8664Gpu => AmiType::Al2023X8664Standard,
        AmiType::BottlerocketX8664Nvidia | AmiType::BottlerocketArm64Nvidia => AmiType::BottlerocketX8664,
        _ => AmiType::Al2023X8664Standard,
      },
      _ => match self.cpu_arch {
        CpuArch::X8664 => AmiType::Al2023X8664Standard,
        CpuArch::Arm64 => AmiType::Al2023X8664Standard,
      },
    };

    // Based on the default AMI type selected, set the default instance type(s) for the default node group
    self.default_instance_types = match self.default_ami_type {
      AmiType::Al2Arm64 | AmiType::Al2X8664 | AmiType::BottlerocketArm64Nvidia => {
        vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()]
      }
      _ => vec!["m7a.xlarge".to_string(), "m7i.xlarge".to_string()],
    };

    Ok(self)
  }
}
