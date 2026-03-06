#![feature(str_as_str)]

pub mod cli;
pub mod config;
pub mod inputs;
pub mod template;
use std::{collections::HashSet, fs, path::Path, str};

use anyhow::{Context, Result};
pub use cli::Cli;
use handlebars::{Handlebars, handlebars_helper};
use rust_embed::RustEmbed;
use serde_json::{Value, value::Map};
use tracing::trace;

/// Embeds the contents of the `templates/` directory into the binary
#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

fn register_handlebars() -> Result<Handlebars<'static>> {
  // Custom helpers
  handlebars_helper!(neq: |v1: Value, v2: Value| v1 != v2);
  handlebars_helper!(eq: |v1: Value, v2: Value| v1 == v2);
  handlebars_helper!(and: |v1: bool, v2: bool| v1 && v2 );
  handlebars_helper!(or: |v1: bool, v2: bool| v1 || v2 );
  handlebars_helper!(snake_case: |v: str| v.replace("-", "_"));

  let mut handlebars = Handlebars::new();
  handlebars.register_helper("neq", Box::new(neq));
  handlebars.register_helper("eq", Box::new(eq));
  handlebars.register_helper("and", Box::new(and));
  handlebars.register_helper("or", Box::new(or));
  handlebars.register_helper("snake_case", Box::new(snake_case));

  // Register templates
  let templates = HashSet::from(["eks.tpl", "helm.tpl", "main.tpl", "variables.tpl"]);
  for tpl in &templates {
    trace!("Registering template: {}", tpl.as_str());
    let embed = Templates::get(tpl.as_str()).ok_or_else(|| anyhow::anyhow!("Missing embedded template: {}", tpl))?;
    let name = tpl.replace(".tpl", "");
    handlebars.register_template_string(&name, std::str::from_utf8(embed.data.as_ref())?)?;
  }

  // Register partials
  for tpl in Templates::iter()
    .filter(|t| !templates.contains(t.as_str()))
    .collect::<Vec<_>>()
  {
    trace!("Registering partial: {}", tpl.as_str());
    let embed = Templates::get(tpl.as_str()).ok_or_else(|| anyhow::anyhow!("Missing embedded partial: {}", tpl))?;
    let name = format!("tpl_{}", tpl.replace(".tpl", "").replace("-", "_"));
    handlebars.register_template_string(&name, std::str::from_utf8(embed.data.as_ref())?)?;
  }

  Ok(handlebars)
}

fn render_template(name: &str, configuration: &inputs::Configuration, handlebars: &Handlebars) -> Result<String> {
  let mut data = Map::new();
  data.insert("inputs".to_string(), handlebars::to_json(configuration));

  handlebars.render(name, &data).map_err(Into::into)
}

const TF_FILES: [&str; 4] = ["eks.tf", "main.tf", "helm.tf", "variables.tf"];

/// Check if the output directory already contains Terraform files.
/// Returns an error listing the existing files if any are found.
pub fn check_existing_tf_files(dir: &Path) -> Result<()> {
  let existing: Vec<_> = TF_FILES.iter().filter(|f| dir.join(f).exists()).collect();
  if !existing.is_empty() {
    anyhow::bail!(
      "Output directory already contains Terraform files: {}. Use --force to overwrite.",
      existing.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ")
    );
  }
  Ok(())
}

fn write_cluster_configs(dir: &Path, configuration: &inputs::Configuration) -> Result<()> {
  let handlebars = crate::register_handlebars()?;

  let eks_path = dir.join("eks.tf");
  fs::write(&eks_path, render_template("eks", configuration, &handlebars)?)
    .with_context(|| format!("Failed to write {}", eks_path.display()))?;
  let main_path = dir.join("main.tf");
  fs::write(&main_path, render_template("main", configuration, &handlebars)?)
    .with_context(|| format!("Failed to write {}", main_path.display()))?;

  let helm = render_template("helm", configuration, &handlebars)?;
  if !helm.is_empty() {
    let helm_path = dir.join("helm.tf");
    fs::write(&helm_path, helm).with_context(|| format!("Failed to write {}", helm_path.display()))?;
  }

  let vars = render_template("variables", configuration, &handlebars)?;
  if !vars.is_empty() {
    let vars_path = dir.join("variables.tf");
    fs::write(&vars_path, vars).with_context(|| format!("Failed to write {}", vars_path.display()))?;
  }

  match std::process::Command::new("terraform").arg("fmt").arg(dir).output() {
    Ok(output) if output.status.success() => tracing::trace!("Terraform files have been formatted"),
    Ok(output) => tracing::warn!("terraform fmt failed: {}", String::from_utf8_lossy(&output.stderr)),
    Err(_) => match std::process::Command::new("tofu").arg("fmt").arg(dir).output() {
      Ok(output) if output.status.success() => tracing::trace!("Terraform files have been formatted with tofu"),
      _ => tracing::info!("Neither terraform nor tofu found in PATH. Skipping formatting."),
    },
  };

  Ok(())
}
