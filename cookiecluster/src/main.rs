use anyhow::Result;
use clap::Parser;
use cookiecluster::{Cli, inputs::Inputs};
use tracing_log::AsTrace;
use tracing_subscriber::FmtSubscriber;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
  let cli = Cli::parse();
  let subscriber = FmtSubscriber::builder()
    .with_max_level(cli.verbose.log_level_filter().as_trace())
    .without_time()
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

  let inputs = Inputs::new().collect()?;
  cli.write(&inputs)
}
