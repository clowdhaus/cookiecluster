#![feature(str_as_str)]

pub mod cli;
pub mod config;
pub mod inputs;
pub mod template;
use std::{collections::HashSet, fs, path::Path, str};

use anyhow::Result;
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
  handlebars_helper!(camel_case: |v: str| v.replace("-", "_"));

  let mut handlebars = Handlebars::new();
  handlebars.register_helper("neq", Box::new(neq));
  handlebars.register_helper("eq", Box::new(eq));
  handlebars.register_helper("and", Box::new(and));
  handlebars.register_helper("or", Box::new(or));
  handlebars.register_helper("camel_case", Box::new(camel_case));

  // Register templates
  let templates = HashSet::from(["eks.tpl", "helm.tpl", "main.tpl", "variables.tpl"]);
  for tpl in &templates {
    trace!("Registering template: {}", tpl.as_str());
    let embed = Templates::get(tpl.as_str()).unwrap();
    let name = tpl.replace(".tpl", "");
    handlebars.register_template_string(&name, std::str::from_utf8(embed.data.as_ref())?)?;
  }

  // Register partials
  for tpl in Templates::iter()
    .filter(|t| !templates.contains(t.as_str()))
    .collect::<Vec<_>>()
  {
    trace!("Registering partial: {}", tpl.as_str());
    let embed = Templates::get(tpl.as_str()).unwrap();
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

fn write_cluster_configs(dir: &Path, configuration: &inputs::Configuration) -> Result<()> {
  let handlebars = crate::register_handlebars()?;

  fs::write(dir.join("eks.tf"), render_template("eks", configuration, &handlebars)?)?;
  fs::write(
    dir.join("main.tf"),
    render_template("main", configuration, &handlebars)?,
  )?;

  let helm = render_template("helm", configuration, &handlebars)?;
  if !helm.is_empty() {
    fs::write(dir.join("helm.tf"), helm)?;
  }

  let vars = render_template("variables", configuration, &handlebars)?;
  if !vars.is_empty() {
    fs::write(dir.join("variables.tf"), vars)?;
  }

  match std::process::Command::new("terraform").arg("fmt").arg(".").output() {
    Ok(_) => tracing::trace!("Terraform files have been formatted"),
    _ => tracing::trace!("Terraform executable not found. Skipping formatting."),
  };

  Ok(())
}
