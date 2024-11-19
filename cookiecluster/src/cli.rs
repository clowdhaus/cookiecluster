use std::{fs, path::Path};

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::{builder::Styles, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use handlebars::Handlebars;
use serde_json::value::Map;

use crate::inputs::Inputs;

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
  pub fn write(self, inputs: &Inputs) -> Result<()> {
    let handlebars = crate::register_handlebars()?;

    fs::write(Path::new("eks.tf"), render_value("eks", inputs, &handlebars)?)?;
    fs::write(
      Path::new("karpenter.tf"),
      render_value("karpenter", inputs, &handlebars)?,
    )?;
    fs::write(Path::new("main.tf"), render_value("main", inputs, &handlebars)?)?;
    fs::write(
      Path::new("variables.tf"),
      render_value("variables", inputs, &handlebars)?,
    )?;
    Ok(())
  }
}

fn render_value(name: &str, inputs: &Inputs, handlebars: &Handlebars) -> Result<String> {
  let mut data = Map::new();
  data.insert("inputs".to_string(), handlebars::to_json(inputs));

  handlebars.render(name, &data).map_err(Into::into)
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::inputs::{add_on, ami, compute, version};

  fn render(inputs: Inputs, dir_name: &str) -> Result<()> {
    let mut settings = insta::Settings::new();
    settings.set_snapshot_path(format!("./snapshots/{dir_name}"));
    let _guard = settings.bind_to_scope();

    for tpl in ["eks", "main", "karpenter", "variables"] {
      let handlebars = crate::register_handlebars()?;
      let rendered = render_value(tpl, &inputs, &handlebars)?;
      insta::assert_snapshot!(tpl, rendered);
    }

    Ok(())
  }

  #[test]
  fn snapshot_default() {
    // Defaults to AL2023
    let inputs = Inputs::default();

    render(inputs, "default").unwrap();
  }

  #[test]
  fn snapshot_al2023_x86_64() {
    let inputs = Inputs {
      ami_type: ami::AmiType::Al2023X8664Standard,
      ..Inputs::default()
    };

    render(inputs, "al2023-x86-64").unwrap();
  }

  #[test]
  fn snapshot_al2023_arm64() {
    let inputs = Inputs {
      ami_type: ami::AmiType::Al2023Arm64Standard,
      instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
      ..Inputs::default()
    };

    render(inputs, "al2032-arm64").unwrap();
  }

  #[test]
  fn snapshot_bottlerocket_x8664() {
    let inputs = Inputs {
      ami_type: ami::AmiType::BottlerocketX8664,
      ..Inputs::default()
    };

    render(inputs, "bottlerocket-x86-64").unwrap();
  }

  #[test]
  fn snapshot_bottlerocket_arm64() {
    let inputs = Inputs {
      ami_type: ami::AmiType::BottlerocketArm64,
      instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
      ..Inputs::default()
    };

    render(inputs, "bottlerocket-arm64").unwrap();
  }

  #[test]
  fn snapshot_nvidia() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::Al2023X8664Nvidia,
      instance_types: vec!["g5.4xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs, "nvidia").unwrap();
  }

  #[test]
  fn snapshot_nvidia_efa() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::Al2023X8664Nvidia,
      enable_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs, "nvidia_efa").unwrap();
  }

  #[test]
  fn snapshot_nvidia_efa_odcr() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::Al2023X8664Nvidia,
      enable_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      reservation: compute::ReservationType::OnDemandCapacityReservation,
      ..Inputs::default()
    };

    render(inputs, "nvidia-efa-odcr").unwrap();
  }

  #[test]
  fn snapshot_nvidia_efa_cbr() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::Al2023X8664Nvidia,
      enable_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      reservation: compute::ReservationType::MlCapacityBlockReservation,
      ..Inputs::default()
    };

    render(inputs, "nvidia-efa-cbr").unwrap();
  }

  #[test]
  fn snapshot_neuron() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      ami_type: ami::AmiType::Al2023X8664Neuron,
      instance_types: vec!["inf2.xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs, "neuron").unwrap();
  }

  #[test]
  fn snapshot_neuron_efa() {
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      ami_type: ami::AmiType::Al2023X8664Neuron,
      enable_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["trn1n.32xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs, "neuron-efa").unwrap();
  }

  #[test]
  fn snapshot_al2023_instance_storage() {
    let inputs = Inputs {
      ami_type: ami::AmiType::Al2023Arm64Standard,
      instance_storage_supported: true,
      instance_types: vec!["m7gd.2xlarge".to_owned()],
      ..Inputs::default()
    };

    render(inputs, "al2023-instance-storage").unwrap();
  }

  #[test]
  fn snapshot_karpenter() {
    let inputs = Inputs {
      compute_scaling: compute::ScalingType::Karpenter,
      ..Inputs::default()
    };

    render(inputs, "karpenter").unwrap();
  }

  #[test]
  fn snapshot_enable_all() {
    let inputs = Inputs {
      cluster_name: "cookiecluster".to_string(),
      cluster_version: version::ClusterVersion::K128,
      cluster_endpoint_public_access: true,
      control_plane_subnet_filter: "*-intra-*".to_string(),
      data_plane_subnet_filter: "*-data-*".to_string(),
      enable_cluster_creator_admin_permissions: true,
      vpc_name: "cookiecluster".to_string(),
      ..Inputs::default()
    };

    render(inputs, "enable-all").unwrap();
  }

  #[test]
  fn snapshot_all_add_ons() {
    let inputs = Inputs {
      add_ons: vec![
        add_on::AddOn {
          name: String::from("coredns"),
          under_name: String::from("coredns"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("kube-proxy"),
          under_name: String::from("kube_proxy"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("vpc-cni"),
          under_name: String::from("vpc-cni"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: Some("aws-node".to_string()),
          },
        },
        add_on::AddOn {
          name: String::from("eks-pod-identity-agent"),
          under_name: String::from("eks_pod_identity_agent"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("aws-ebs-csi-driver"),
          under_name: String::from("aws_ebs_csi_driver"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: Some("module.aws_ebs_csi_driver_pod_identity.iam_role_arn".to_string()),
            pod_identity_service_account: Some("ebs-csi-controller-sa".to_string()),
          },
        },
        add_on::AddOn {
          name: String::from("aws-efs-csi-driver"),
          under_name: String::from("aws_efs_csi_driver"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: Some("module.aws_efs_csi_driver_pod_identity.iam_role_arn".to_string()),
            pod_identity_service_account: Some("efs-csi-controller-sa".to_string()),
          },
        },
        add_on::AddOn {
          name: String::from("aws-mountpoint-s3-csi-driver"),
          under_name: String::from("aws_mountpoint_s3_csi_driver"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: Some("module.aws_mountpoint_s3_csi_driver_pod_identity.iam_role_arn".to_string()),
            pod_identity_service_account: Some("s3-csi-driver-sa".to_string()),
          },
        },
        add_on::AddOn {
          name: String::from("snapshot-controller"),
          under_name: String::from("snapshot_controller"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("adot"),
          under_name: String::from("adot"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("aws-guardduty-agent"),
          under_name: String::from("aws_guardduty_agent"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: None,
            pod_identity_service_account: None,
          },
        },
        add_on::AddOn {
          name: String::from("amazon-cloudwatch-observability"),
          under_name: String::from("amazon_cloudwatch_observability"),
          configuration: add_on::AddOnConfiguration {
            pod_identity_role_arn: Some("module.amazon_cloudwatch_observability_pod_identity.iam_role_arn".to_string()),
            pod_identity_service_account: Some("cloudwatch-agent".to_string()),
          },
        },
      ],
      ..Inputs::default()
    };

    render(inputs, "all-add-ons").unwrap();
  }
}
