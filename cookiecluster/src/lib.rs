#![feature(str_as_str)]

pub mod cli;
pub mod inputs;

use anyhow::Result;
pub use cli::Cli;
use handlebars::{handlebars_helper, Handlebars};
use rust_embed::RustEmbed;
use serde_json::Value;

/// Embeds the contents of the `templates/` directory into the binary
#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

fn register_handlebars() -> Result<Handlebars<'static>> {
  // Custom helpers
  handlebars_helper!(eq: |v1: Value, v2: Value| v1 == v2);
  handlebars_helper!(and: |v1: bool, v2: bool| v1 && v2 );
  handlebars_helper!(or: |v1: bool, v2: bool| v1 || v2 );

  let mut handlebars = Handlebars::new();
  handlebars.register_helper("eq", Box::new(eq));
  handlebars.register_helper("and", Box::new(and));
  handlebars.register_helper("or", Box::new(or));

  for tpl in Templates::iter().collect::<Vec<_>>() {
    let rtpl = Templates::get(tpl.as_str()).unwrap();
    handlebars.register_template_string(&tpl.replace(".tpl", ""), std::str::from_utf8(rtpl.data.as_ref())?)?;
  }

  Ok(handlebars)
}
