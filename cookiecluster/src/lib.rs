#![feature(str_as_str)]

pub mod cli;
pub mod inputs;

use std::{collections::HashSet, str};

use anyhow::Result;
pub use cli::Cli;
use handlebars::{Handlebars, handlebars_helper};
use rust_embed::RustEmbed;
use serde_json::Value;
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
  let templates = HashSet::from(["eks.tpl", "karpenter.tpl", "main.tpl", "variables.tpl"]);
  for tpl in &templates {
    trace!("Registering template: {}", tpl.as_str());
    let embed: rust_embed::EmbeddedFile = Templates::get(tpl.as_str()).unwrap();
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
