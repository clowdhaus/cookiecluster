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
  // Files that will be generated
  // eks.tf - from addons.tpl, cluster.tpl, node-group.tpl, node-group-accel.tpl, and pod-identity.tpl
  // helm.tf - from device-plugins.tpl
  // karpenter.tf - from karpenter.tpl
  // main.tf - from main.tpl, network.tpl
  // variables.tf - from variables.tpl
  pub fn write(self, inputs: &Inputs) -> Result<()> {
    let handlebars = crate::register_handlebars()?;

    fs::write(
      Path::new("eks.tf"),
      render_value(
        "eks",
        &["addons", "node-groups", "node-group-accel", "pod-identity"],
        inputs,
        &handlebars,
      )?,
    )?;
    // fs::write(
    //   Path::new("helm.tf"),
    //   render_value("helm", &["device-plugins"], inputs, &handlebars)?,
    // )?;
    fs::write(
      Path::new("karpenter.tf"),
      render_value("karpenter", &["karpenter"], inputs, &handlebars)?,
    )?;
    fs::write(
      Path::new("main.tf"),
      render_value("main", &["main", "network"], inputs, &handlebars)?,
    )?;
    fs::write(
      Path::new("variables.tf"),
      render_value("variables", &["variables"], inputs, &handlebars)?,
    )?;
    Ok(())
  }
}

fn render_value(name: &str, templates: &[&str], inputs: &Inputs, handlebars: &Handlebars) -> Result<String> {
  let mut data = Map::new();
  data.insert("inputs".to_string(), handlebars::to_json(inputs));

  for tpl in templates {
    let rendered = handlebars.render(tpl, &data)?;
    data.insert(format!("tpl_{tpl}"), handlebars::to_json(&rendered));
  }

  handlebars.render(name, &data).map_err(Into::into)
}

// #[cfg(test)]
// mod tests {

//   use super::*;
//   use crate::inputs::{add_on, ami, compute, version};

//   #[test]
//   fn snapshot_default() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     // Defaults to AL2023
//     let inputs = Inputs::default();
//     let rendered = cli.render(&inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_al2_x8664() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       ami_type: ami::AmiType::Al2X8664,
//       ..Inputs::default()
//     };
//     let rendered = cli.render(&inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_al2_arm64() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       ami_type: ami::AmiType::Al2Arm64,
//       instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_bottlerocket_x8664() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       ami_type: ami::AmiType::BottlerocketX8664,
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_bottlerocket_arm64() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       ami_type: ami::AmiType::BottlerocketArm64,
//       instance_types: vec!["m7g.xlarge".to_string(), "m6g.xlarge".to_string()],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_nvidia() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       accelerator: compute::AcceleratorType::Nvidia,
//       ami_type: ami::AmiType::Al2X8664Gpu,
//       instance_types: vec!["g5.4xlarge".to_owned()],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_nvidia_efa() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       accelerator: compute::AcceleratorType::Nvidia,
//       ami_type: ami::AmiType::Al2X8664Gpu,
//       enable_efa: true,
//       instance_storage_supported: true,
//       instance_types: vec!["p5.48xlarge".to_owned()],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_nvidia_efa_odcr() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       accelerator: compute::AcceleratorType::Nvidia,
//       ami_type: ami::AmiType::Al2X8664Gpu,
//       enable_efa: true,
//       instance_storage_supported: true,
//       instance_types: vec!["p5.48xlarge".to_owned()],
//       reservation: compute::ReservationType::OnDemandCapacityReservation,
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_nvidia_efa_cbr() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       accelerator: compute::AcceleratorType::Nvidia,
//       ami_type: ami::AmiType::Al2X8664Gpu,
//       enable_efa: true,
//       instance_storage_supported: true,
//       instance_types: vec!["p5.48xlarge".to_owned()],
//       reservation: compute::ReservationType::MlCapacityBlockReservation,
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_neuron() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       accelerator: compute::AcceleratorType::Neuron,
//       ami_type: ami::AmiType::Al2X8664Gpu,
//       instance_types: vec!["inf2.xlarge".to_owned()],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_neuron_efa() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       accelerator: compute::AcceleratorType::Neuron,
//       ami_type: ami::AmiType::Al2X8664Gpu,
//       enable_efa: true,
//       instance_storage_supported: true,
//       instance_types: vec!["trn1n.32xlarge".to_owned()],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_al2_instance_storage() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       ami_type: ami::AmiType::Al2Arm64,
//       instance_storage_supported: true,
//       instance_types: vec!["m7gd.2xlarge".to_owned()],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_al2023_instance_storage() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       ami_type: ami::AmiType::Al2023Arm64Standard,
//       instance_storage_supported: true,
//       instance_types: vec!["m7gd.2xlarge".to_owned()],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_karpenter() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       compute_scaling: compute::ScalingType::Karpenter,
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_enable_all() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       cluster_name: "cookiecluster".to_string(),
//       cluster_version: version::ClusterVersion::K128,
//       cluster_endpoint_public_access: true,
//       control_plane_subnet_filter: "*-intra-*".to_string(),
//       data_plane_subnet_filter: "*-data-*".to_string(),
//       enable_cluster_creator_admin_permissions: true,
//       vpc_name: "cookiecluster".to_string(),
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }

//   #[test]
//   fn snapshot_all_addons() {
//     let cli = Cli {
//       verbose: Verbosity::default(),
//     };
//     let inputs = Inputs {
//       add_ons: vec![
//         add_on::AddOn {
//           name: String::from("coredns"),
//           under_name: String::from("coredns"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: None,
//           },
//         },
//         add_on::AddOn {
//           name: String::from("kube-proxy"),
//           under_name: String::from("kube_proxy"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: None,
//           },
//         },
//         add_on::AddOn {
//           name: String::from("vpc-cni"),
//           under_name: String::from("vpc-cni"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: None,
//           },
//         },
//         add_on::AddOn {
//           name: String::from("eks-pod-identity-agent"),
//           under_name: String::from("eks_pod_identity_agent"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: None,
//           },
//         },
//         add_on::AddOn {
//           name: String::from("aws-ebs-csi-driver"),
//           under_name: String::from("aws_ebs_csi_driver"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: Some("module.aws_ebs_csi_driver_irsa.iam_role_arn".to_string()),
//           },
//         },
//         add_on::AddOn {
//           name: String::from("aws-efs-csi-driver"),
//           under_name: String::from("aws_efs_csi_driver"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: Some("module.aws_efs_csi_driver_irsa.iam_role_arn".to_string()),
//           },
//         },
//         add_on::AddOn {
//           name: String::from("aws-mountpoint-s3-csi-driver"),
//           under_name: String::from("aws_mountpoint_s3_csi_driver"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: Some("module.aws_mountpoint_s3_csi_driver_irsa.iam_role_arn".to_string()),
//           },
//         },
//         add_on::AddOn {
//           name: String::from("snapshot-controller"),
//           under_name: String::from("snapshot_controller"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: None,
//           },
//         },
//         add_on::AddOn {
//           name: String::from("adot"),
//           under_name: String::from("adot"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: None,
//           },
//         },
//         add_on::AddOn {
//           name: String::from("aws-guardduty-agent"),
//           under_name: String::from("aws_guardduty_agent"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: None,
//           },
//         },
//         add_on::AddOn {
//           name: String::from("amazon-cloudwatch-observability"),
//           under_name: String::from("amazon_cloudwatch_observability"),
//           configuration: add_on::AddOnConfiguration {
//             service_account_role_arn: Some("module.amazon_cloudwatch_observability_irsa.iam_role_arn".to_string()),
//           },
//         },
//       ],
//       ..Inputs::default()
//     };
//     let rendered = cli.render(inputs).unwrap();
//     insta::assert_snapshot!(rendered);
//   }
// }
