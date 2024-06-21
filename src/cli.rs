use anstyle::{AnsiColor, Color, Style};
use clap::{builder::Styles, Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
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
  #[command(subcommand)]
  pub command: Commands,

  #[clap(flatten)]
  pub verbose: Verbosity<InfoLevel>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
  /// ToDo
  DoSomething(Something),
}

#[derive(Args, Debug, Deserialize, Serialize)]
pub struct Something {
  /// ToDo
  #[clap(short, long)]
  this: String,
}
