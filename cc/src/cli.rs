use std::{fs, path::Path};

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::{builder::Styles, Parser, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

/// Styles for CLI
fn get_styles() -> Styles {
  Styles::styled()
    .header(
      Style::new()
        .bold()
        .underline()
        .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
    )
    .literal(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
    .usage(
      Style::new()
        .bold()
        .underline()
        .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
    )
    .placeholder(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Magenta))))
}

#[derive(Debug, Parser)]
#[command(author, about, version)]
#[command(propagate_version = true)]
#[command(styles=get_styles())]
pub struct Cli {
  /// The template to use
  #[arg(short, long, value_enum, default_value_t)]
  tmpl: Tmpl,

  #[clap(flatten)]
  pub verbose: Verbosity<InfoLevel>,
}

#[derive(Debug, Clone, ValueEnum)]
enum Tmpl {
  Standard,
  Hpc,
  Ml,
}

impl Default for Tmpl {
  fn default() -> Self {
    Self::Standard
  }
}

impl Cli {
  pub fn write(self, inputs: crate::Inputs) -> Result<()> {
    let template = crate::Templates::get("eks.tpl").unwrap();

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("tpl", std::str::from_utf8(template.data.as_ref())?)?;

    let rendered = handlebars.render("tpl", &inputs)?;
    fs::write(Path::new("eks.tf"), rendered)?;

    Ok(())
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct Eks {
  name: String,
  region: String,
  node_type: String,
  node_count: u32,
}
