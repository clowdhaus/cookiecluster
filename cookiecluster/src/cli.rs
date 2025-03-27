use std::{fs, path::Path};

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::{Parser, builder::Styles};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use handlebars::Handlebars;
use serde_json::value::Map;

use crate::inputs::Output;

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
  pub fn write(self, output: &Output) -> Result<()> {
    let handlebars = crate::register_handlebars()?;

    fs::write(Path::new("eks.tf"), render_value("eks", output, &handlebars)?)?;
    fs::write(Path::new("main.tf"), render_value("main", output, &handlebars)?)?;

    let helm = render_value("helm", output, &handlebars)?;
    if !helm.is_empty() {
      fs::write(Path::new("helm.tf"), helm)?;
    }

    let vars = render_value("variables", output, &handlebars)?;
    if !vars.is_empty() {
      fs::write(Path::new("variables.tf"), vars)?;
    }

    match std::process::Command::new("terraform").arg("fmt").arg(".").output() {
      Ok(_) => tracing::trace!("Terraform files have been formatted"),
      _ => tracing::trace!("Terraform executable not found. Skipping formatting."),
    };

    Ok(())
  }
}

fn render_value(name: &str, output: &Output, handlebars: &Handlebars) -> Result<String> {
  let mut data = Map::new();
  data.insert("inputs".to_string(), handlebars::to_json(output));

  handlebars.render(name, &data).map_err(Into::into)
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::inputs::{Inputs, add_on, ami, compute, version};

  fn render(output: Output, dir_name: &str) -> Result<()> {
    let mut settings = insta::Settings::new();
    settings.set_snapshot_path(format!("./snapshots/{dir_name}"));
    let _guard = settings.bind_to_scope();

    for tpl in ["eks", "main", "helm", "variables"] {
      let handlebars = crate::register_handlebars()?;
      let rendered = render_value(tpl, &output, &handlebars)?;
      insta::assert_snapshot!(tpl, rendered);
    }

    Ok(())
  }

  #[test]
  fn snapshot_default() {
    // Defaults to AL2023
    let inputs = Inputs::default();

    render(inputs.to_output(), "default").unwrap();
  }

  #[test]
  fn snapshot_al2023_x86_64() {
    let inputs = Inputs {
      ami_type: ami::AmiType::AL2023_x86_64_STANDARD,
      compute_scaling: compute::ScalingType::None,
      ..Inputs::default()
    };

    render(inputs.to_output(), "al2023-x86-64").unwrap();
  }

  #[test]
  fn snapshot_al2023_arm64() {
    let inputs = Inputs {
      ami_type: ami::AmiType::AL2023_ARM_64_STANDARD,
      compute_scaling: compute::ScalingType::None,
      instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
      ..Inputs::default()
    };

    render(inputs.to_output(), "al2032-arm64").unwrap();
  }

  #[test]
  fn snapshot_bottlerocket_x8664() {
    let inputs = Inputs {
      ami_type: ami::AmiType::BOTTLEROCKET_x86_64,
      compute_scaling: compute::ScalingType::None,
      ..Inputs::default()
    };

    render(inputs.to_output(), "bottlerocket-x86-64").unwrap();
  }

  #[test]
  fn snapshot_bottlerocket_arm64() {
    let inputs = Inputs {
      ami_type: ami::AmiType::BOTTLEROCKET_ARM_64,
      compute_scaling: compute::ScalingType::None,
      instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
      ..Inputs::default()
    };

    render(inputs.to_output(), "bottlerocket-arm64").unwrap();
  }

  #[test]
  fn snapshot_nvidia() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::AL2023_x86_64_NVIDIA,
      compute_scaling: compute::ScalingType::None,
      instance_types: vec!["g5.4xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs.to_output(), "nvidia").unwrap();
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

    render(inputs.to_output(), "nvidia_efa").unwrap();
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

    render(inputs.to_output(), "nvidia-efa-odcr").unwrap();
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

    render(inputs.to_output(), "nvidia-efa-cbr").unwrap();
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

    render(inputs.to_output(), "nvidia-cbr").unwrap();
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

    render(inputs.to_output(), "neuron").unwrap();
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

    render(inputs.to_output(), "neuron-efa").unwrap();
  }

  #[test]
  fn snapshot_neuron_cbr() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      ami_type: ami::AmiType::AL2023_x86_64_NEURON,
      compute_scaling: compute::ScalingType::None,
      require_efa: false,
      instance_storage_supported: true,
      instance_types: vec!["trn1.32xlarge".to_owned()],
      reservation: compute::ReservationType::MlCapacityBlockReservation,
      ..Inputs::default()
    };

    render(inputs.to_output(), "neuron-cbr").unwrap();
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

    render(inputs.to_output(), "al2023-instance-storage").unwrap();
  }

  #[test]
  fn snapshot_karpenter() {
    let inputs = Inputs {
      compute_scaling: compute::ScalingType::Karpenter,
      ..Inputs::default()
    };

    render(inputs.to_output(), "karpenter").unwrap();
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

    render(inputs.to_output(), "karpenter-odcr").unwrap();
  }

  #[test]
  fn snapshot_auto_mode() {
    let inputs = Inputs {
      compute_scaling: compute::ScalingType::AutoMode,
      add_ons: vec![],
      ..Inputs::default()
    };

    render(inputs.to_output(), "auto-mode").unwrap();
  }

  #[test]
  fn snapshot_enable_all() {
    let inputs = Inputs {
      cluster_name: "cookiecluster".to_string(),
      cluster_version: version::ClusterVersion::K128,
      cluster_endpoint_public_access: true,
      compute_scaling: compute::ScalingType::None,
      control_plane_subnet_filter: "*-intra-*".to_string(),
      data_plane_subnet_filter: "*-data-*".to_string(),
      enable_cluster_creator_admin_permissions: true,
      vpc_name: "cookiecluster".to_string(),
      ..Inputs::default()
    };

    render(inputs.to_output(), "enable-all").unwrap();
  }

  #[test]
  fn snapshot_all_add_ons() {
    let inputs = Inputs {
      add_ons: add_on::_get_all_add_ons(),
      compute_scaling: compute::ScalingType::None,
      ..Inputs::default()
    };

    render(inputs.to_output(), "all-add-ons").unwrap();
  }
}
