use std::{fs::File, path::{Path, PathBuf}};
use std::io::prelude::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::inputs::Inputs;

#[derive(Debug, Serialize, Deserialize)]
struct ClusterSpec {
  name: String,
  description: String,
  target_dir: PathBuf,
  params: Inputs,
}

fn load_cluster_specifications(config_path: &Path) -> Result<Vec<ClusterSpec>> {
  let config = config_path.canonicalize().expect("Failed to canonicalize config path");
  let file = std::fs::File::open(config.as_os_str())?;
  let specs: Vec<ClusterSpec> = serde_yaml::from_reader(file)?;
  tracing::trace!("Loaded cluster specifications: {:#?}", specs);

  Ok(specs)
}

pub fn generate_cluster_configurations(config_path: &Path) -> Result<()> {
  let specs = load_cluster_specifications(config_path)?;

  for spec in specs {
    tracing::trace!("Generating cluster spec: {}", spec.name);

    let target_dir = spec.target_dir.canonicalize().unwrap_or_else(|_| spec.target_dir.clone());
    let unique_dir = target_dir.join(&spec.name);

    if !unique_dir.exists() {
      std::fs::create_dir_all(&unique_dir)?;
    }

    crate::write_cluster_configs(&unique_dir, &spec.params.to_configuration())?;

    let mut readme = File::create(&unique_dir.join("README.md"))?;
    readme.write_all(&spec.description.as_bytes())?;
  }

  Ok(())
}
