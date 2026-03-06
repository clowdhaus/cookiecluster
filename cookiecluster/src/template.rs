use std::path::Path;

use anyhow::Result;

use crate::config;

fn load_embedded_templates() -> Result<config::Configuration> {
  let embed = crate::Templates::get("clusters.yaml")
    .ok_or_else(|| anyhow::anyhow!("Missing embedded clusters.yaml template"))?;
  let config: config::Configuration = serde_yaml_ng::from_slice(embed.data.as_ref())?;
  Ok(config)
}

fn get_starter_templates(config: &config::Configuration) -> Vec<String> {
  config
    .clusters
    .iter()
    .map(|cluster| cluster.name.to_lowercase())
    .collect()
}

pub fn list_templates() -> Result<()> {
  let config = load_embedded_templates()?;
  let templates = get_starter_templates(&config);
  println!("Available templates:");
  for name in &templates {
    println!("  {name}");
  }
  Ok(())
}

pub fn generate_from_template(template_name: &str, output_dir: &Path) -> Result<()> {
  let config = load_embedded_templates()?;
  let templates = get_starter_templates(&config);
  if !templates.contains(&template_name.to_lowercase()) {
    let available = templates.join(", ");
    return Err(anyhow::anyhow!(
      "Template '{}' not found.\nAvailable templates: {}",
      template_name,
      available
    ));
  }

  for spec in config.clusters {
    if spec.name.to_lowercase() != template_name.to_lowercase() {
      continue;
    }

    tracing::trace!("Generating cluster spec: {}", spec.name);
    let inputs = match spec.params {
      Some(params) => config::update_default_inputs(params, spec.name.clone())?,
      None => crate::inputs::Inputs::default(),
    };
    crate::write_cluster_configs(output_dir, &inputs.to_configuration()?)?;
    println!("Generated Terraform files in {}", std::fs::canonicalize(output_dir)?.display());
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_load_embedded_templates() {
    let config = load_embedded_templates().unwrap();
    assert!(!config.clusters.is_empty());
  }

  #[test]
  fn test_get_starter_templates_returns_lowercase() {
    let config = load_embedded_templates().unwrap();
    let templates = get_starter_templates(&config);
    assert!(templates.contains(&"default".to_string()));
    for name in &templates {
      assert_eq!(name, &name.to_lowercase());
    }
  }

  #[test]
  fn test_generate_from_template_invalid_name() {
    let dir = std::env::temp_dir();
    let result = generate_from_template("nonexistent-template", &dir);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found"));
    assert!(err.contains("Available templates"));
  }

  #[test]
  fn test_generate_from_template_default() {
    let dir = tempfile::tempdir().unwrap();
    generate_from_template("default", dir.path()).unwrap();
    assert!(dir.path().join("eks.tf").exists());
    assert!(dir.path().join("main.tf").exists());
  }

  #[test]
  fn test_check_existing_tf_files_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    assert!(crate::check_existing_tf_files(dir.path()).is_ok());
  }

  #[test]
  fn test_check_existing_tf_files_blocks_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("eks.tf"), "# existing").unwrap();
    let result = crate::check_existing_tf_files(dir.path());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("eks.tf"));
    assert!(err.contains("--force"));
  }
}
