use std::path::{Path, PathBuf};

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::{Parser, builder::Styles};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use crate::inputs;

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

  /// Path to a configuration file in YAML format
  #[clap(long)]
  pub config: Option<PathBuf>,
}

impl Cli {
  pub fn write_cluster_configs(self, configuration: &inputs::Configuration) -> Result<()> {
    let dir = Path::new(".");
    crate::write_cluster_configs(dir, configuration)
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::inputs::{Inputs, add_on, ami, compute};

  fn render(configuration: inputs::Configuration, dir_name: &str) -> Result<()> {
    let mut settings = insta::Settings::new();
    settings.set_snapshot_path(format!("./snapshots/{dir_name}"));
    let _guard = settings.bind_to_scope();

    for tpl in ["eks", "main", "helm", "variables"] {
      let handlebars = crate::register_handlebars()?;
      let rendered = crate::render_template(tpl, &configuration, &handlebars)?;
      insta::assert_snapshot!(tpl, rendered);
    }

    Ok(())
  }

  #[test]
  fn snapshot_default() {
    // Defaults to AL2023
    let inputs = Inputs::default();

    render(inputs.to_configuration(), "default").unwrap();
  }

  #[test]
  fn snapshot_al2023_x86_64() {
    let inputs = Inputs {
      compute_scaling: compute::ScalingType::None,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "al2023-x86-64").unwrap();
  }

  #[test]
  fn snapshot_al2023_arm64() {
    let inputs = Inputs {
      ami_type: ami::AmiType::AL2023_ARM_64_STANDARD,
      compute_scaling: compute::ScalingType::None,
      instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "al2032-arm64").unwrap();
  }

  #[test]
  fn snapshot_bottlerocket_x8664() {
    let inputs = Inputs {
      ami_type: ami::AmiType::BOTTLEROCKET_x86_64,
      compute_scaling: compute::ScalingType::None,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "bottlerocket-x86-64").unwrap();
  }

  #[test]
  fn snapshot_bottlerocket_arm64() {
    let inputs = Inputs {
      ami_type: ami::AmiType::BOTTLEROCKET_ARM_64,
      compute_scaling: compute::ScalingType::None,
      instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "bottlerocket-arm64").unwrap();
  }

  #[test]
  fn snapshot_nvidia() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::AL2023_x86_64_NVIDIA,
      compute_scaling: compute::ScalingType::None,
      instance_types: vec!["g6e.2xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "nvidia").unwrap();
  }

  #[test]
  fn snapshot_nvidia_efa() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::AL2023_x86_64_NVIDIA,
      compute_scaling: compute::ScalingType::None,
      require_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "nvidia_efa").unwrap();
  }

  #[test]
  fn snapshot_nvidia_efa_odcr() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::AL2023_x86_64_NVIDIA,
      compute_scaling: compute::ScalingType::None,
      require_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      reservation: compute::ReservationType::OnDemandCapacityReservation,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "nvidia-efa-odcr").unwrap();
  }

  #[test]
  fn snapshot_nvidia_efa_cbr() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::AL2023_x86_64_NVIDIA,
      compute_scaling: compute::ScalingType::None,
      require_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      reservation: compute::ReservationType::MlCapacityBlockReservation,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "nvidia-efa-cbr").unwrap();
  }

  #[test]
  fn snapshot_nvidia_cbr() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::AL2023_x86_64_NVIDIA,
      compute_scaling: compute::ScalingType::None,
      require_efa: false,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      reservation: compute::ReservationType::MlCapacityBlockReservation,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "nvidia-cbr").unwrap();
  }

  #[test]
  fn snapshot_neuron() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      ami_type: ami::AmiType::AL2023_x86_64_NEURON,
      compute_scaling: compute::ScalingType::None,
      instance_types: vec!["inf2.xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "neuron").unwrap();
  }

  #[test]
  fn snapshot_neuron_efa() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      ami_type: ami::AmiType::AL2023_x86_64_NEURON,
      compute_scaling: compute::ScalingType::None,
      require_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["trn1n.32xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "neuron-efa").unwrap();
  }

  #[test]
  fn snapshot_neuron_cbr() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      ami_type: ami::AmiType::AL2023_x86_64_NEURON,
      compute_scaling: compute::ScalingType::None,
      require_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["trn1.32xlarge".to_owned()],
      reservation: compute::ReservationType::MlCapacityBlockReservation,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "neuron-cbr").unwrap();
  }

  #[test]
  fn snapshot_al2023_instance_storage() {
    let inputs = Inputs {
      ami_type: ami::AmiType::AL2023_ARM_64_STANDARD,
      compute_scaling: compute::ScalingType::None,
      instance_storage_supported: true,
      instance_types: vec!["m7gd.2xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "al2023-instance-storage").unwrap();
  }

  #[test]
  fn snapshot_karpenter() {
    let inputs = Inputs {
      compute_scaling: compute::ScalingType::Karpenter,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "karpenter").unwrap();
  }

  #[test]
  fn snapshot_karpenter_odcr() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::AL2023_x86_64_NVIDIA,
      compute_scaling: compute::ScalingType::Karpenter,
      instance_storage_supported: true,
      instance_types: vec!["p5e.48xlarge".to_owned()],
      reservation: compute::ReservationType::OnDemandCapacityReservation,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "karpenter-odcr").unwrap();
  }

  #[test]
  fn snapshot_auto_mode() {
    let inputs = Inputs {
      compute_scaling: compute::ScalingType::AutoMode,
      add_on_types: vec![],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "auto-mode").unwrap();
  }

  #[test]
  fn snapshot_auto_mode_nvidia() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      compute_scaling: compute::ScalingType::AutoMode,
      add_on_types: vec![],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "auto-mode-nvidia").unwrap();
  }

  #[test]
  fn snapshot_auto_mode_neuron() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      compute_scaling: compute::ScalingType::AutoMode,
      add_on_types: vec![],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "auto-mode-neuron").unwrap();
  }

  #[test]
  fn snapshot_auto_mode_efa() {
    let inputs = Inputs {
      require_efa: true,
      compute_scaling: compute::ScalingType::AutoMode,
      add_on_types: vec![],
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "auto-mode-efa").unwrap();
  }

  #[test]
  fn snapshot_enable_all() {
    let inputs = Inputs {
      name: "cookiecluster".to_string(),
      endpoint_public_access: true,
      compute_scaling: compute::ScalingType::None,
      control_plane_subnet_filter: "*-intra-*".to_string(),
      data_plane_subnet_filter: "*-data-*".to_string(),
      enable_cluster_creator_admin_permissions: true,
      vpc_name: "cookiecluster".to_string(),
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "enable-all").unwrap();
  }

  #[test]
  fn snapshot_all_add_ons() {
    let inputs = Inputs {
      add_on_types: add_on::_get_all_add_on_types(),
      compute_scaling: compute::ScalingType::None,
      ..Inputs::default()
    };

    render(inputs.to_configuration(), "all-add-ons").unwrap();
  }
}
