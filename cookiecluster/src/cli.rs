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
  fn render(self, inputs: crate::inputs::Inputs) -> Result<String> {
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

    Ok(cluster_rendered)
  }

  pub fn write(self, inputs: crate::inputs::Inputs) -> Result<()> {
    let cluster_rendered = self.render(inputs)?;
    fs::write(Path::new("eks.tf"), cluster_rendered).map_err(Into::into)
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::inputs::{add_on, ami, compute, version, Inputs};

  #[test]
  fn snapshot_default() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    // Defaults to AL2023
    let inputs = Inputs::default();
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_al2_x8664() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      ami_type: ami::AmiType::Al2X8664,
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_al2_arm64() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      ami_type: ami::AmiType::Al2Arm64,
      instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_bottlerocket_x8664() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      ami_type: ami::AmiType::BottlerocketX8664,
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_bottlerocket_arm64() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      ami_type: ami::AmiType::BottlerocketArm64,
      instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_nvidia() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::Al2X8664Gpu,
      instance_types: vec!["g5.4xlarge".to_owned()],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_nvidia_efa() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::Al2X8664Gpu,
      enable_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_nvidia_efa_odcr() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::Al2X8664Gpu,
      enable_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      reservation: compute::ReservationType::OnDemandCapacityReservation,
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_nvidia_efa_cbr() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Nvidia,
      ami_type: ami::AmiType::Al2X8664Gpu,
      enable_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["p5.48xlarge".to_owned()],
      reservation: compute::ReservationType::MlCapacityBlockReservation,
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }


  #[test]
  fn snapshot_neuron() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      ami_type: ami::AmiType::Al2X8664Gpu,
      instance_types: vec!["inf2.xlarge".to_owned()],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_neuron_efa() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      accelerator: compute::AcceleratorType::Neuron,
      ami_type: ami::AmiType::Al2X8664Gpu,
      enable_efa: true,
      instance_storage_supported: true,
      instance_types: vec!["trn1n.32xlarge".to_owned()],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_al2_instance_storage() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      ami_type: ami::AmiType::Al2Arm64,
      instance_storage_supported: true,
      instance_types: vec!["m7gd.2xlarge".to_owned()],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_al2023_instance_storage() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      ami_type: ami::AmiType::Al2023Arm64Standard,
      instance_storage_supported: true,
      instance_types: vec!["m7gd.2xlarge".to_owned()],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_karpenter() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      compute_scaling: compute::ScalingType::Karpenter,
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_enable_all() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
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
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }

  #[test]
  fn snapshot_all_addons() {
    let cli = Cli {
      verbose: Verbosity::default(),
    };
    let inputs = Inputs {
      add_ons: vec![
        add_on::AddOn {
          name: String::from("coredns"),
          under_name: String::from("coredns"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: None,
          },
        },
        add_on::AddOn {
          name: String::from("kube-proxy"),
          under_name: String::from("kube_proxy"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: None,
          },
        },
        add_on::AddOn {
          name: String::from("vpc-cni"),
          under_name: String::from("vpc-cni"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: None,
          },
        },
        add_on::AddOn {
          name: String::from("eks-pod-identity-agent"),
          under_name: String::from("eks_pod_identity_agent"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: None,
          },
        },
        add_on::AddOn {
          name: String::from("aws-ebs-csi-driver"),
          under_name: String::from("aws_ebs_csi_driver"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: Some("module.aws_ebs_csi_driver_irsa.iam_role_arn".to_string()),
          },
        },
        add_on::AddOn {
          name: String::from("aws-efs-csi-driver"),
          under_name: String::from("aws_efs_csi_driver"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: Some("module.aws_efs_csi_driver_irsa.iam_role_arn".to_string()),
          },
        },
        add_on::AddOn {
          name: String::from("aws-mountpoint-s3-csi-driver"),
          under_name: String::from("aws_mountpoint_s3_csi_driver"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: Some("module.aws_mountpoint_s3_csi_driver_irsa.iam_role_arn".to_string()),
          },
        },
        add_on::AddOn {
          name: String::from("snapshot-controller"),
          under_name: String::from("snapshot_controller"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: None,
          },
        },
        add_on::AddOn {
          name: String::from("adot"),
          under_name: String::from("adot"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: None,
          },
        },
        add_on::AddOn {
          name: String::from("aws-guardduty-agent"),
          under_name: String::from("aws_guardduty_agent"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: None,
          },
        },
        add_on::AddOn {
          name: String::from("amazon-cloudwatch-observability"),
          under_name: String::from("amazon_cloudwatch_observability"),
          configuration: add_on::AddOnConfiguration {
            service_account_role_arn: Some("module.amazon_cloudwatch_observability_irsa.iam_role_arn".to_string()),
          },
        },
      ],
      ..Inputs::default()
    };
    let rendered = cli.render(inputs).unwrap();
    insta::assert_snapshot!(rendered);
  }
}
