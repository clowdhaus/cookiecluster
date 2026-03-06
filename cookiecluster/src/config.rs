use std::{
  fs::File,
  io::prelude::*,
  path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::inputs::{Inputs, add_on, ami, compute, validate, version};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Configuration {
  defaults: DefaultInputs,
  pub(crate) clusters: Vec<ClusterSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DefaultInputs {
  target_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConfigInputs {
  accelerator: Option<compute::AcceleratorType>,
  add_on_types: Option<Vec<add_on::AddOnType>>,
  ami_type: Option<ami::AmiType>,
  endpoint_public_access: Option<bool>,
  kubernetes_version: Option<version::ClusterVersion>,
  control_plane_subnet_filter: Option<String>,
  compute_scaling: Option<compute::ScalingType>,
  cpu_arch: Option<compute::CpuArch>,
  data_plane_subnet_filter: Option<String>,
  enable_cluster_creator_admin_permissions: Option<bool>,
  require_efa: Option<bool>,
  instance_types: Option<Vec<String>>,
  reservation: Option<compute::ReservationType>,
  reservation_availability_zone: Option<String>,
  vpc_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ClusterSpec {
  pub(crate) name: String,
  description: String,
  target_dir: Option<PathBuf>,
  pub(crate) params: Option<ConfigInputs>,
}

pub(crate) fn load_cluster_specifications(config_path: &Path) -> Result<Configuration> {
  let path = config_path
    .canonicalize()
    .with_context(|| format!("Failed to resolve config path: {}", config_path.display()))?;
  let file = std::fs::File::open(path.as_os_str())
    .with_context(|| format!("Failed to open config file: {}", path.display()))?;
  let config: Configuration = serde_yaml_ng::from_reader(file).context("Failed to parse YAML config file")?;
  tracing::trace!("Loaded configuration file: {:#?}", config);

  Ok(config)
}

pub fn generate_cluster_configurations(config_path: &Path) -> Result<()> {
  let config = load_cluster_specifications(config_path)?;

  for spec in config.clusters {
    tracing::trace!("Generating cluster spec: {}", spec.name);

    let target_dir = spec
      .target_dir
      .or_else(|| config.defaults.target_dir.clone())
      .with_context(|| format!("Target directory must be specified for cluster '{}'", spec.name))?
      .canonicalize()
      .with_context(|| format!("Target directory does not exist for cluster '{}'", spec.name))?;
    let unique_dir = target_dir.join(&spec.name);
    anyhow::ensure!(
      unique_dir.starts_with(&target_dir),
      "Cluster name '{}' would create a path outside the target directory",
      spec.name
    );
    if !unique_dir.exists() {
      std::fs::create_dir_all(&unique_dir)
        .with_context(|| format!("Failed to create directory: {}", unique_dir.display()))?;
    }

    let inputs = match spec.params {
      Some(params) => update_default_inputs(params, spec.name.clone())?,
      None => Inputs::default(),
    };
    crate::write_cluster_configs(&unique_dir, &inputs.to_configuration()?)?;

    let mut readme = File::create(unique_dir.join("README.md"))?;
    readme.write_all(spec.description.as_bytes())?;

    println!("Generated cluster '{}' in {}", spec.name, unique_dir.display());
  }

  Ok(())
}

// Takes user defined parameters and updates the default inputs accordingly
pub(crate) fn update_default_inputs(params: ConfigInputs, name: String) -> Result<Inputs> {
  let mut inputs = Inputs::default();

  if let Some(accelerator) = params.accelerator {
    inputs.accelerator = accelerator;
  }
  if let Some(add_on_types) = params.add_on_types {
    inputs.add_on_types = add_on_types;
  }
  if let Some(ami_type) = params.ami_type {
    inputs.ami_type = ami_type;
  }
  if let Some(endpoint_public_access) = params.endpoint_public_access {
    inputs.endpoint_public_access = endpoint_public_access;
  }
  let name = name.to_lowercase();
  validate::name(&name).map_err(|e| anyhow::anyhow!("Invalid cluster name '{}': {}", name, e))?;
  inputs.name = name;
  if let Some(kubernetes_version) = params.kubernetes_version {
    inputs.kubernetes_version = kubernetes_version;
  }
  if let Some(control_plane_subnet_filter) = params.control_plane_subnet_filter {
    validate::filter(&control_plane_subnet_filter)
      .map_err(|e| anyhow::anyhow!("Invalid control_plane_subnet_filter '{}': {}", control_plane_subnet_filter, e))?;
    inputs.control_plane_subnet_filter = control_plane_subnet_filter;
  }
  if let Some(compute_scaling) = params.compute_scaling {
    inputs.compute_scaling = compute_scaling;
  }
  if let Some(cpu_arch) = params.cpu_arch {
    inputs.cpu_arch = cpu_arch;
  }
  if let Some(data_plane_subnet_filter) = params.data_plane_subnet_filter {
    validate::filter(&data_plane_subnet_filter)
      .map_err(|e| anyhow::anyhow!("Invalid data_plane_subnet_filter '{}': {}", data_plane_subnet_filter, e))?;
    inputs.data_plane_subnet_filter = data_plane_subnet_filter;
  }
  if let Some(enable_cluster_creator_admin_permissions) = params.enable_cluster_creator_admin_permissions {
    inputs.enable_cluster_creator_admin_permissions = enable_cluster_creator_admin_permissions;
  }
  if let Some(require_efa) = params.require_efa {
    inputs.require_efa = require_efa;
  }
  if let Some(instance_types) = params.instance_types {
    for it in &instance_types {
      validate::instance_type(it)
        .map_err(|e| anyhow::anyhow!("Invalid instance type '{}': {}", it, e))?;
    }
    inputs.instance_types = instance_types;
  }
  inputs.instance_storage_supported =
    crate::inputs::compute::instance_storage_supported(&inputs.instance_types, &inputs.ami_type);
  if let Some(reservation) = params.reservation {
    inputs.reservation = reservation;
  }
  if let Some(reservation_availability_zone) = params.reservation_availability_zone {
    validate::availability_zone(&reservation_availability_zone)
      .map_err(|e| anyhow::anyhow!("Invalid reservation_availability_zone '{}': {}", reservation_availability_zone, e))?;
    inputs.reservation_availability_zone = reservation_availability_zone;
  }
  if let Some(vpc_name) = params.vpc_name {
    validate::name(&vpc_name)
      .map_err(|e| anyhow::anyhow!("Invalid vpc_name '{}': {}", vpc_name, e))?;
    inputs.vpc_name = vpc_name;
  }

  Ok(inputs)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_update_default_inputs_basic() {
    let params = ConfigInputs {
      accelerator: Some(compute::AcceleratorType::Nvidia),
      ami_type: Some(ami::AmiType::AL2023_x86_64_NVIDIA),
      endpoint_public_access: Some(true),
      compute_scaling: Some(compute::ScalingType::None),
      instance_types: Some(vec!["g6e.2xlarge".to_string()]),
      add_on_types: None,
      kubernetes_version: None,
      control_plane_subnet_filter: None,
      cpu_arch: None,
      data_plane_subnet_filter: None,
      enable_cluster_creator_admin_permissions: None,
      require_efa: None,
      reservation: None,
      reservation_availability_zone: None,
      vpc_name: None,
    };
    let inputs = update_default_inputs(params, "test-cluster".to_string()).unwrap();
    assert_eq!(inputs.accelerator, compute::AcceleratorType::Nvidia);
    assert_eq!(inputs.ami_type, ami::AmiType::AL2023_x86_64_NVIDIA);
    assert!(inputs.endpoint_public_access);
    assert_eq!(inputs.name, "test-cluster");
    assert_eq!(inputs.instance_types, vec!["g6e.2xlarge"]);
  }

  #[test]
  fn test_update_default_inputs_invalid_name() {
    let params = ConfigInputs {
      accelerator: None,
      add_on_types: None,
      ami_type: None,
      endpoint_public_access: None,
      kubernetes_version: None,
      control_plane_subnet_filter: None,
      compute_scaling: None,
      cpu_arch: None,
      data_plane_subnet_filter: None,
      enable_cluster_creator_admin_permissions: None,
      require_efa: None,
      instance_types: None,
      reservation: None,
      reservation_availability_zone: None,
      vpc_name: None,
    };
    let result = update_default_inputs(params, "../../bad".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn test_update_default_inputs_invalid_instance_type() {
    let params = ConfigInputs {
      accelerator: None,
      add_on_types: None,
      ami_type: None,
      endpoint_public_access: None,
      kubernetes_version: None,
      control_plane_subnet_filter: None,
      compute_scaling: None,
      cpu_arch: None,
      data_plane_subnet_filter: None,
      enable_cluster_creator_admin_permissions: None,
      require_efa: None,
      instance_types: Some(vec!["${file(\"/etc/shadow\")}".to_string()]),
      reservation: None,
      reservation_availability_zone: None,
      vpc_name: None,
    };
    let result = update_default_inputs(params, "valid-name".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn test_update_default_inputs_defaults_preserved() {
    let params = ConfigInputs {
      accelerator: None,
      add_on_types: None,
      ami_type: None,
      endpoint_public_access: None,
      kubernetes_version: None,
      control_plane_subnet_filter: None,
      compute_scaling: None,
      cpu_arch: None,
      data_plane_subnet_filter: None,
      enable_cluster_creator_admin_permissions: None,
      require_efa: None,
      instance_types: None,
      reservation: None,
      reservation_availability_zone: None,
      vpc_name: None,
    };
    let inputs = update_default_inputs(params, "test".to_string()).unwrap();
    // Should retain defaults
    assert_eq!(inputs.kubernetes_version, version::ClusterVersion::K134);
    assert_eq!(inputs.accelerator, compute::AcceleratorType::None);
    assert!(!inputs.endpoint_public_access);
  }

  #[test]
  fn test_load_cluster_specifications_missing_file() {
    let result = load_cluster_specifications(Path::new("/nonexistent/path.yaml"));
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Failed to resolve config path"));
  }
}
