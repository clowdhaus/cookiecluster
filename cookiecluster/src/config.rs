use std::path::Path;

use anyhow::Result;
use config::Config;

use crate::inputs::Inputs;

pub fn get_configs(config_path: &Path) -> Result<()> {
  let canonical_config_path = config_path.canonicalize().expect("Failed to canonicalize config path");
  let config_path_str = canonical_config_path
    .as_os_str()
    .to_str()
    .expect("Failed to convert config path to string");

  let settings = Config::builder()
    .set_default("accelerator", "")?
    .add_source(config::File::new(config_path_str, config::FileFormat::Yaml))
    .build()?;

  let configs: Vec<Inputs> = settings
    .try_deserialize()
    .expect("Failed to deserialize inputs from config");

  println!("Loaded configurations: {:#?}", configs);

  Ok(())
}
