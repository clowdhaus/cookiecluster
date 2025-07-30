use std::{
  fs::File,
  io::prelude::*,
  path::{Path, PathBuf},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::inputs::{Inputs, add_on, ami, compute, version};

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
  let path = config_path.canonicalize().expect("Failed to canonicalize config path");
  let file = std::fs::File::open(path.as_os_str())?;
  let config: Configuration = serde_yaml::from_reader(file)?;
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
      .expect("Target directory must be specified")
      .canonicalize()
      .unwrap();
    let unique_dir = target_dir.join(&spec.name);
    if !unique_dir.exists() {
      std::fs::create_dir_all(&unique_dir)?;
    }

    let inputs = match spec.params {
      Some(params) => update_default_inputs(params, spec.name.clone())?,
      None => Inputs::default(),
    };
    crate::write_cluster_configs(&unique_dir, &inputs.to_configuration())?;

    let mut readme = File::create(unique_dir.join("README.md"))?;
    readme.write_all(spec.description.as_bytes())?;
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
  inputs.name = name.to_lowercase();
  if let Some(kubernetes_version) = params.kubernetes_version {
    inputs.kubernetes_version = kubernetes_version;
  }
  if let Some(control_plane_subnet_filter) = params.control_plane_subnet_filter {
    inputs.control_plane_subnet_filter = control_plane_subnet_filter;
  }
  if let Some(compute_scaling) = params.compute_scaling {
    inputs.compute_scaling = compute_scaling;
  }
  if let Some(cpu_arch) = params.cpu_arch {
    inputs.cpu_arch = cpu_arch;
  }
  if let Some(data_plane_subnet_filter) = params.data_plane_subnet_filter {
    inputs.data_plane_subnet_filter = data_plane_subnet_filter;
  }
  if let Some(enable_cluster_creator_admin_permissions) = params.enable_cluster_creator_admin_permissions {
    inputs.enable_cluster_creator_admin_permissions = enable_cluster_creator_admin_permissions;
  }
  if let Some(require_efa) = params.require_efa {
    inputs.require_efa = require_efa;
  }
  if let Some(instance_types) = params.instance_types {
    inputs.instance_types = instance_types;
  }
  inputs.instance_storage_supported =
    crate::inputs::compute::instance_storage_supported(&inputs.instance_types, &inputs.ami_type);
  if let Some(reservation) = params.reservation {
    inputs.reservation = reservation;
  }
  if let Some(reservation_availability_zone) = params.reservation_availability_zone {
    inputs.reservation_availability_zone = reservation_availability_zone;
  }
  if let Some(vpc_name) = params.vpc_name {
    inputs.vpc_name = vpc_name;
  }

  Ok(inputs)
}
