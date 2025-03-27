pub(crate) mod add_on;
pub(crate) mod ami;
pub(crate) mod compute;
pub(crate) mod instance;
pub(crate) mod version;

pub use add_on::AddOn;
use anyhow::Result;
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};

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
  pub(crate) require_efa: bool,
  pub(crate) instance_storage_supported: bool,
  pub(crate) instance_types: Vec<String>,
  pub(crate) reservation: compute::ReservationType,
  pub(crate) reservation_availability_zone: String,
  pub(crate) vpc_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  enable_accelerator: bool,
  enable_nvidia_gpus: bool,
  enable_neuron_devices: bool,
  enable_efa: bool,

  enable_add_ons: bool,
  enable_pod_identity: bool,
  enable_helm: bool,
  enable_public_ecr_helm: bool,

  enable_auto_mode: bool,
  enable_karpenter: bool,
  enable_compute_reservation: bool,
  enable_odcr: bool,
  enable_ml_cbr: bool,

  // Pass through
  add_ons: Vec<add_on::AddOn>,
  ami_type: ami::AmiType,
  cluster_endpoint_public_access: bool,
  cluster_name: String,
  cluster_version: version::ClusterVersion,
  control_plane_subnet_filter: String,
  cpu_arch: compute::CpuArch,
  data_plane_subnet_filter: String,
  default_ami_type: ami::AmiType,
  default_instance_types: Vec<String>,
  enable_cluster_creator_admin_permissions: bool,
  instance_storage_supported: bool,
  instance_types: Vec<String>,
  reservation_availability_zone: String,
  vpc_name: String,
}

impl Default for Inputs {
  fn default() -> Self {
    Inputs {
      accelerator: compute::AcceleratorType::None,
      add_ons: add_on::get_default_add_ons(),
      ami_type: ami::AmiType::AL2023_x86_64_STANDARD,
      cluster_endpoint_public_access: false,
      cluster_name: String::from("example"),
      cluster_version: version::ClusterVersion::K132,
      control_plane_subnet_filter: String::from("*-private-*"),
      compute_scaling: compute::ScalingType::AutoMode,
      cpu_arch: compute::CpuArch::X8664,
      data_plane_subnet_filter: String::from("*-private-*"),
      default_ami_type: ami::AmiType::AL2023_x86_64_STANDARD,
      default_instance_types: vec!["m7a.xlarge".to_string(), "m7i.xlarge".to_string()],
      enable_cluster_creator_admin_permissions: false,
      require_efa: false,
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

  pub fn collect(self) -> Result<Output> {
    let inputs = self
      .collect_cluster_settings()?
      .collect_accelerator_type()?
      .collect_require_efa()?
      .collect_reservation_type()?
      .collect_compute_scaling_type()?
      .collect_add_ons()?
      .collect_networking_settings()?
      .collect_cpu_arch()?
      .collect_ami_type()?
      .collect_instance_types()?
      .collect_storage_settings()?
      .collect_default_node_group_settings()?;

    let outputs = inputs.to_output();

    Ok(outputs)
  }

  fn collect_cluster_settings(mut self) -> Result<Inputs> {
    self.cluster_name = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Cluster name")
      .interact_text()?;

    let cluster_versions = version::ClusterVersion::versions();
    let idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Cluster version")
      .items(&cluster_versions[..])
      .default(0)
      .interact()?;
    self.cluster_version = version::ClusterVersion::from_idx(idx);

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

  fn collect_accelerator_type(mut self) -> Result<Inputs> {
    let all_accelerators = compute::get_accelerator_types();
    let idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Accelerator type")
      .items(&all_accelerators[..])
      .default(0)
      .interact()?;
    self.accelerator = compute::AcceleratorType::from_idx(idx);

    Ok(self)
  }

  fn collect_require_efa(mut self) -> Result<Inputs> {
    match self.accelerator {
      compute::AcceleratorType::Nvidia | compute::AcceleratorType::Neuron => {
        self.require_efa = Confirm::with_theme(&ColorfulTheme::default())
          .with_prompt("Enable EFA support")
          .default(true)
          .interact()?
      }
      _ => {}
    }

    Ok(self)
  }

  fn collect_reservation_type(mut self) -> Result<Inputs> {
    let reservation_types = compute::get_reservation_types(&self.accelerator);
    let idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("EC2 capacity reservation")
      .items(&reservation_types[..])
      .default(0)
      .interact()?;
    self.reservation = compute::ReservationType::from_idx(idx);

    Ok(self)
  }

  fn collect_compute_scaling_type(mut self) -> Result<Inputs> {
    let scaling_types = compute::get_compute_scaling_types(&self.reservation);
    let idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Compute autoscaling")
      .items(&scaling_types[..])
      .default(0)
      .interact()?;
    self.compute_scaling = compute::ScalingType::from_idx(idx);

    Ok(self)
  }

  fn collect_add_ons(mut self) -> Result<Inputs> {
    let all_add_ons = add_on::get_add_on_names(&self.compute_scaling);
    let add_ons_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("EKS add-on(s)")
      .items(&all_add_ons[..])
      .defaults(&add_on::get_default_add_on_flags())
      .interact()?;

    let add_ons = add_ons_idxs
      .iter()
      .map(|&i| add_on::get_add_on(add_on::AddOnType::from_idx(i)).unwrap())
      .collect::<Vec<add_on::AddOn>>();
    self.add_ons = add_ons;

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

    if self.reservation != compute::ReservationType::None && self.compute_scaling != compute::ScalingType::Karpenter {
      self.reservation_availability_zone = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("EC2 capacity reservation availability zone")
        .with_initial_text("us-west-2a".to_string())
        .interact_text()?;
    }

    Ok(self)
  }

  fn collect_cpu_arch(mut self) -> Result<Inputs> {
    if !should_collect_arch(&self.compute_scaling, &self.accelerator, self.require_efa) {
      return Ok(self);
    }

    let idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("CPU architecture")
      .item("x86-64")
      .item("arm64")
      .default(0)
      .interact()?;
    self.cpu_arch = compute::CpuArch::from_idx(idx);

    Ok(self)
  }

  fn collect_ami_type(mut self) -> Result<Inputs> {
    if self.compute_scaling == compute::ScalingType::AutoMode {
      return Ok(self);
    }

    let ami_types = ami::get_ami_types(&self.accelerator, self.require_efa, &self.cpu_arch);
    let idx = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("AMI type")
      .items(&ami_types[..])
      .default(0)
      .interact()?;
    self.ami_type = ami::AmiType::from_idx(idx);

    Ok(self)
  }

  fn collect_instance_types(mut self) -> Result<Inputs> {
    if self.compute_scaling == compute::ScalingType::AutoMode {
      return Ok(self);
    }

    let instance_type_names =
      compute::get_instance_type_names(&self.cpu_arch, self.require_efa, &self.accelerator, &self.reservation);

    let instance_idxs = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Instance type(s)")
      .items(&instance_type_names)
      .interact()?;

    self.instance_types =
      compute::limit_instances_selected(&self.reservation, self.require_efa, instance_type_names, instance_idxs)?;

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
      ami::AmiType::AL2023_ARM_64_STANDARD
      | ami::AmiType::BOTTLEROCKET_ARM_64
      | ami::AmiType::BOTTLEROCKET_ARM_64_NVIDIA => {
        vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()]
      }
      _ => vec!["m7a.xlarge".to_string(), "m7i.xlarge".to_string()],
    };

    Ok(self)
  }

  pub fn to_output(self) -> Output {
    Output {
      enable_accelerator: self.accelerator != compute::AcceleratorType::None,
      enable_nvidia_gpus: self.accelerator == compute::AcceleratorType::Nvidia,
      enable_neuron_devices: self.accelerator == compute::AcceleratorType::Neuron,
      enable_efa: self.require_efa,

      enable_add_ons: !self.add_ons.is_empty(),
      enable_pod_identity: self
        .add_ons
        .iter()
        .any(|a| a.configuration.is_some() && a.configuration.as_ref().unwrap().pod_identity_role_arn.is_some()),
      enable_helm: should_enable_helm(&self.accelerator, &self.compute_scaling, self.require_efa),
      enable_public_ecr_helm: should_enable_public_ecr_helm(&self.accelerator, &self.compute_scaling),

      enable_auto_mode: self.compute_scaling == compute::ScalingType::AutoMode,
      enable_karpenter: self.compute_scaling == compute::ScalingType::Karpenter,
      enable_compute_reservation: self.reservation != compute::ReservationType::None,
      enable_odcr: self.reservation == compute::ReservationType::OnDemandCapacityReservation,
      enable_ml_cbr: self.reservation == compute::ReservationType::MlCapacityBlockReservation,

      // Pass through
      add_ons: self.add_ons,
      ami_type: self.ami_type,
      cluster_endpoint_public_access: self.cluster_endpoint_public_access,
      cluster_name: self.cluster_name,
      cluster_version: self.cluster_version,
      control_plane_subnet_filter: self.control_plane_subnet_filter,
      cpu_arch: self.cpu_arch,
      data_plane_subnet_filter: self.data_plane_subnet_filter,
      default_ami_type: self.default_ami_type,
      default_instance_types: self.default_instance_types,
      enable_cluster_creator_admin_permissions: self.enable_cluster_creator_admin_permissions,
      instance_storage_supported: self.instance_storage_supported,
      instance_types: self.instance_types,
      reservation_availability_zone: self.reservation_availability_zone,
      vpc_name: self.vpc_name,
    }
  }
}

fn should_collect_arch(
  scaling_type: &compute::ScalingType,
  accelerator: &compute::AcceleratorType,
  require_efa: bool,
) -> bool {
  // Set on Auto Mode/Karpenter NodeClass
  if *scaling_type == compute::ScalingType::Karpenter || *scaling_type == compute::ScalingType::AutoMode {
    return false;
  }

  // Inf/Trn instances only support x86-64 at this time
  if *accelerator == compute::AcceleratorType::Neuron {
    return false;
  }

  if *accelerator == compute::AcceleratorType::Nvidia && require_efa {
    return false;
  }

  true
}

// Standard Helm resources are:
// - NVIDIA device plugin
// - EFA device plugin
fn should_enable_helm(
  accelerator: &compute::AcceleratorType,
  compute: &compute::ScalingType,
  require_efa: bool,
) -> bool {
  // Auto Mode bundles the NVIDIA device plugin and EFA device plugin
  if compute == &compute::ScalingType::AutoMode {
    return false;
  }
  if accelerator == &compute::AcceleratorType::Nvidia || require_efa {
    return true;
  }
  false
}

// Public ECR Helm resources are:
// - Neuron Helm chart
// - Karpenter Helm chart
fn should_enable_public_ecr_helm(accelerator: &compute::AcceleratorType, compute: &compute::ScalingType) -> bool {
  // Auto Mode bundles the Neuron Helm chart and Karpenter Helm chart (equivalents)
  if compute == &compute::ScalingType::AutoMode {
    return false;
  }
  if accelerator == &compute::AcceleratorType::Neuron || compute == &compute::ScalingType::Karpenter {
    return true;
  }
  false
}
