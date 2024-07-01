use std::{fs, path::Path, str::from_utf8};

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::{builder::Styles, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use handlebars::{handlebars_helper, Handlebars};
use serde_json::{value::Map, Value};

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

/// Cookiecluster - A CLI to generate EKS cluster definitions in Terraform.
///
/// Based on a few inputs, cookiecluster will guide you through the process of designing a cluster that fits your
/// requirements using the appropriate values. It does not require any AWS credentials; it is merely a glorified
/// templating engine for EKS clusters defined in Terraform.
#[derive(Debug, Parser)]
#[command(author, about, version)]
#[command(propagate_version = true)]
#[command(styles=get_styles())]
pub struct Cli {
  #[clap(flatten)]
  pub verbose: Verbosity<InfoLevel>,
}

impl Cli {
  pub fn write(self, inputs: crate::Inputs) -> Result<()> {
    let cluster_tpl = crate::Templates::get("cluster.tpl").unwrap();
    let accelerated_mng_tpl = crate::Templates::get("accel-mng.tpl").unwrap();

    handlebars_helper!(eq: |v1: Value, v2: Value| v1 == v2);
    handlebars_helper!(and: |v1: bool, v2: bool| v1 && v2 );
    handlebars_helper!(or: |v1: bool, v2: bool| v1 || v2 );

    let mut handlebars = Handlebars::new();
    handlebars.register_helper("eq", Box::new(eq));
    handlebars.register_helper("and", Box::new(and));
    handlebars.register_helper("or", Box::new(or));
    handlebars.register_template_string("cluster", from_utf8(cluster_tpl.data.as_ref())?)?;
    handlebars.register_template_string("accelerated_mng", from_utf8(accelerated_mng_tpl.data.as_ref())?)?;

    let mut data = Map::new();
    // Handlebars prefers json/maps instead of nested rust data types
    data.insert("add_ons".to_string(), handlebars::to_json(&inputs.add_ons));
    data.insert("inputs".to_string(), handlebars::to_json(&inputs));

    let accelerated_mng_rendered = handlebars.render("accelerated_mng", &data)?;
    data.insert(
      "accelerated_mng".to_string(),
      handlebars::to_json(accelerated_mng_rendered),
    );

    let cluster_rendered = handlebars.render("cluster", &data)?;
    fs::write(Path::new("eks.tf"), cluster_rendered)?;

    Ok(())
  }
}
