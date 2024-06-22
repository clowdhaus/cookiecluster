use anyhow::Result;
use clap::Parser;
use cookiecluster::{cli::Cli, instances};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use tracing_log::AsTrace;
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
  let cli = Cli::parse();
  let subscriber = FmtSubscriber::builder()
    .with_max_level(cli.verbose.log_level_filter().as_trace())
    .without_time()
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

  let type_indices = MultiSelect::with_theme(&ColorfulTheme::default())
    .with_prompt("Instance type(s")
    .items(instances::INSTANCE_TYPES)
    // .defaults(&defaults[..])
    .interact()?;

  let selections = type_indices
    .iter()
    .map(|&i| instances::INSTANCE_TYPES[i].to_string())
    .collect::<Vec<String>>();
  println!("{:#?}", selections);

  cli.write()
}
