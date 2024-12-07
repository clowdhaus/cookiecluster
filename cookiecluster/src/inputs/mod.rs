pub(crate) mod add_on;
pub(crate) mod ami;
pub(crate) mod compute;
pub(crate) mod instance;
pub(crate) mod version;

pub use add_on::AddOn;
use anyhow::Result;
use compute::AcceleratorType;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[derive(Debug, Serialize, Deserialize)]
pub struct Inputs {
  pub(crate) accelerator: compute::AcceleratorType,
  pub(crate) add_ons: Vec<add_on::AddOn>,
  pub(crate) ami_type: ami::AmiType,
  pub(crate) cluster_endpoint_public_access: bool,
  pub(crate) cluster_name: String,
  pub(crate) cluster_version: version::ClusterVersion,
  pub(crate) control_plane_subnet_filter: String,
  pub(crate) compute_scaling: compute::ScalingType,
  pub(crate) cpu_arch: compute::CpuArch,
  pub(crate) data_plane_subnet_filter: String,
  /// AMI type used on default node group when secondary node group (accelerated, Windows, etc) is used
  pub(crate) default_ami_type: ami::AmiType,
  /// Instance types used on default node group when secondary node group (accelerated, Windows, etc) is used
  pub(crate) default_instance_types: Vec<String>,
  pub(crate) enable_cluster_creator_admin_permissions: bool,
  pub(crate) enable_efa: bool,
  pub(crate) instance_storage_supported: bool,
  pub(crate) instance_types: Vec<String>,
  pub(crate) reservation: compute::ReservationType,
  pub(crate) reservation_availability_zone: String,
  pub(crate) vpc_name: String,
}

impl Default for Inputs {
  fn default() -> Self {
    Inputs {
      accelerator: compute::AcceleratorType::None,
      add_ons: vec![
        add_on::AddOn {
          name: String::from("coredns"),
          under_name: String::from("coredns"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("eks-pod-identity-agent"),
          under_name: String::from("eks_pod_identity_agent"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("kube-proxy"),
          under_name: String::from("kube_proxy"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("vpc-cni"),
          under_name: String::from("vpc_cni"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: Some("module.vpc_cni_pod_identity.iam_role_arn".to_string()),
            pod_identity_service_account: Some("aws-node".to_string()),
          },
        },
      ],
      ami_type: ami::AmiType::Al2023X8664Standard,
      cluster_endpoint_public_access: false,
      cluster_name: String::from("example"),
      cluster_version: version::ClusterVersion::K131,
      control_plane_subnet_filter: String::from("*-private-*"),
      compute_scaling: compute::ScalingType::AutoMode,
      cpu_arch: compute::CpuArch::X8664,
      data_plane_subnet_filter: String::from("*-private-*"),
      default_ami_type: ami::AmiType::Al2023X8664Standard,
      default_instance_types: vec!["m7a.xlarge".to_string(), "m7i.xlarge".to_string()],
      enable_cluster_creator_admin_permissions: false,
      enable_efa: false,
      instance_storage_supported: false,
      instance_types: vec!["m7a.xlarge".to_string(), "m7i.xlarge".to_string()],
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
      .collect_accelerator_type()?
      .collect_enable_efa()?
      .collect_reservation_type()?
      .collect_compute_scaling_type()?
      .collect_add_ons()?
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
    if self.compute_scaling == compute::ScalingType::AutoMode {
      self.add_ons = vec![];
      return Ok(self);
    }

    let all_add_ons = add_on::get_add_ons()?;
    let add_ons_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("EKS add-on(s)")
      .items(&all_add_ons[..])
      // Select first 4 add-ons by default
      .defaults(&[true, true, true, true])
      .interact()?;

    let add_ons = add_ons_idxs
      .iter()
      .map(|&i| add_on::get_add_on_configuration(all_add_ons[i].as_str()).unwrap())
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
    if self.accelerator == AcceleratorType::None {
      return Ok(self);
    }

    let reservation_types = compute::get_reservation_types(&self.accelerator);

    let reservation_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("EC2 capacity reservation")
      .items(&reservation_types[..])
      .default(0)
      .interact()?;

    self.reservation = compute::ReservationType::from(reservation_types[reservation_idx]);

    Ok(self)
  }

  fn collect_compute_scaling_type(mut self) -> Result<Inputs> {
    let scaling_types = compute::get_scaling_types(&self.reservation);

    let compute_scaling_idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Compute autoscaling")
      .items(&scaling_types[..])
      .default(0)
      .interact()?;

    self.compute_scaling = compute::ScalingType::from(scaling_types[compute_scaling_idx]);

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
    // Set on Auto Mode/Karpenter NodeClass
    if self.compute_scaling == compute::ScalingType::Karpenter || self.compute_scaling == compute::ScalingType::AutoMode
    {
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
    if self.compute_scaling == compute::ScalingType::AutoMode {
      return Ok(self);
    }

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
    if self.compute_scaling == compute::ScalingType::AutoMode {
      return Ok(self);
    }

    let instance_type_names =
      compute::get_instance_type_names(&self.cpu_arch, self.enable_efa, &self.accelerator, &self.reservation);

    let instance_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Instance type(s)")
      .items(&instance_type_names)
      .interact()?;

    self.instance_types =
      compute::limit_instances_selected(&self.reservation, self.enable_efa, instance_type_names, instance_idxs)?;

    Ok(self)
  }

  fn collect_storage_settings(mut self) -> Result<Inputs> {
    self.instance_storage_supported =
      compute::instance_storage_supported(self.instance_types.as_slice(), &self.ami_type);

    Ok(self)
  }

  fn collect_default_node_group_settings(mut self) -> Result<Inputs> {
    self.default_ami_type = ami::get_default_ami_type(&self.ami_type, &self.cpu_arch);

    // Based on the default AMI type selected, set the default instance type(s) for the default node group
    self.default_instance_types = match self.default_ami_type {
      ami::AmiType::Al2023X8664Neuron | ami::AmiType::Al2023X8664Nvidia | ami::AmiType::BottlerocketArm64Nvidia => {
        vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()]
      }
      _ => vec!["m7a.xlarge".to_string(), "m7i.xlarge".to_string()],
    };

    Ok(self)
  }
}
