use anyhow::Result;
use clap::Parser;
use cookiecluster::cli as cc;
use tracing_log::AsTrace;
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
  let cli = cc::Cli::parse();
  let subscriber = FmtSubscriber::builder()
    .with_max_level(cli.verbose.log_level_filter().as_trace())
    .without_time()
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

  Ok(())
}
