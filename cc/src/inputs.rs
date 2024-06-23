use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};

use crate::INSTANCE_TYPES;

#[derive(Debug)]
pub struct Inputs {
  instance_types: Vec<String>,
  workload_type: WorkloadType,
}

impl Default for Inputs {
  fn default() -> Self {
    Inputs {
      instance_types: vec![],
      workload_type: WorkloadType::Standard,
    }
  }
}

#[derive(Debug)]
enum WorkloadType {
  Standard,
  HpcWithEfa,
  MlWithEfa,
  Ml,
}

impl Inputs {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn collect(self) -> Result<Self> {
    let inputs = self.collect_workload_type()?.collect_instance_types()?;

    Ok(inputs)
  }

  fn collect_workload_type(mut self) -> Result<Inputs> {
    let workload_type_index = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Workload type")
      .item("Standard")
      .item("HPC with EFA")
      .item("ML with EFA")
      .item("ML")
      .default(0)
      .interact()?;

    let workload_type = match workload_type_index {
      1 => WorkloadType::HpcWithEfa,
      2 => WorkloadType::MlWithEfa,
      3 => WorkloadType::Ml,
      _ => WorkloadType::Standard,
    };
    self.workload_type = workload_type;

    Ok(self)
  }

  fn collect_instance_types(mut self) -> Result<Inputs> {
    let instance_type_indices = MultiSelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Instance type(s)")
      .items(INSTANCE_TYPES)
      .interact()?;

    let instance_types = instance_type_indices
      .iter()
      .map(|&i| INSTANCE_TYPES[i].to_string())
      .collect::<Vec<String>>();
    self.instance_types = instance_types;

    Ok(self)
  }
}
