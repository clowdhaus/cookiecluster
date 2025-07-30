use std::path::Path;

use anyhow::Result;

use crate::config;

fn load_embedded_templates() -> Result<config::Configuration> {
  let embed = crate::Templates::get("clusters.yaml").unwrap();
  let config: config::Configuration = serde_yaml::from_slice(embed.data.as_ref())?;
  Ok(config)
}

fn get_starter_templates(config: &config::Configuration) -> Result<Vec<String>> {
  let templates = config
    .clusters
    .iter()
    .map(|cluster| cluster.name.to_lowercase())
    .collect::<Vec<String>>();

  Ok(templates)
}

pub fn generate_from_template(template_name: &str) -> Result<()> {
  let config = load_embedded_templates()?;
  let templates = get_starter_templates(&config)?;
  if !templates.contains(&template_name.to_string()) {
    return Err(anyhow::anyhow!(
      "Template '{}' not found.\nAvailable templates: {:#?}",
      template_name,
      templates
    ));
  }

  for spec in config.clusters {
    tracing::trace!("Generating cluster spec: {}", spec.name);

    if spec.name.to_lowercase() != template_name.to_lowercase() {
      continue;
    }

    let inputs = match spec.params {
      Some(params) => config::update_default_inputs(params, spec.name.clone())?,
      None => crate::inputs::Inputs::default(),
    };
    crate::write_cluster_configs(Path::new("."), &inputs.to_configuration())?;
  }

  Ok(())
}
