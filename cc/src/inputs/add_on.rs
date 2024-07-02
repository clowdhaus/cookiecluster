use std::fmt;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddOnType {
  CoreDns,
  KubeProxy,
  VpcCni,
  EksPodIdentityAgent,
  AwsEbsCsiDriver,
  AwsEfsCsiDriver,
  AwsMountpointS3CsiDriver,
  SnapshotController,
  Adot,
  AwsGuarddutyAgent,
  AmazonCloudwatchObservability,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AddOn {
  pub name: String,
  pub under_name: String,
  pub configuration: AddOnConfiguration,
}

/// Get all available add-ons
pub fn get_add_ons() -> Result<Vec<String>> {
  Ok(AddOnType::iter().map(|v| v.to_string()).collect::<Vec<_>>())
}

/// For a given add-on, return (required) configuration
pub fn get_add_on_configuration(name: &str) -> Result<AddOn> {
  let add_on_type = AddOnType::from(name);
  let add_on = match add_on_type {
    // Not adding vpc-cni since it still requires permissions on node IAM role to start
    AddOnType::AwsEbsCsiDriver
    | AddOnType::AwsEfsCsiDriver
    | AddOnType::AwsMountpointS3CsiDriver
    | AddOnType::AmazonCloudwatchObservability => {
      let under_name = add_on_type.to_string().replace('-', "_");
      AddOn {
        name: add_on_type.to_string(),
        under_name: under_name.to_string(),
        configuration: AddOnConfiguration {
          service_account_role_arn: Some(format!("module.{under_name}_irsa.iam_role_arn")),
        },
      }
    }
    _ => AddOn {
      name: add_on_type.to_string(),
      under_name: add_on_type.to_string().replace('-', "_"),
      configuration: AddOnConfiguration {
        service_account_role_arn: None,
      },
    },
  };

  Ok(add_on)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AddOnConfiguration {
  pub service_account_role_arn: Option<String>,
}

impl std::fmt::Display for AddOnType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AddOnType::CoreDns => write!(f, "coredns"),
      AddOnType::KubeProxy => write!(f, "kube-proxy"),
      AddOnType::VpcCni => write!(f, "vpc-cni"),
      AddOnType::EksPodIdentityAgent => write!(f, "eks-pod-identity-agent"),
      AddOnType::AwsEbsCsiDriver => write!(f, "aws-ebs-csi-driver"),
      AddOnType::AwsEfsCsiDriver => write!(f, "aws-efs-csi-driver"),
      AddOnType::AwsMountpointS3CsiDriver => write!(f, "aws-mountpoint-s3-csi-driver"),
      AddOnType::SnapshotController => write!(f, "snapshot-controller"),
      AddOnType::Adot => write!(f, "adot"),
      AddOnType::AwsGuarddutyAgent => write!(f, "aws-guardduty-agent"),
      AddOnType::AmazonCloudwatchObservability => write!(f, "amazon-cloudwatch-observability"),
    }
  }
}

impl std::convert::From<&str> for AddOnType {
  fn from(s: &str) -> Self {
    match s {
      "coredns" => AddOnType::CoreDns,
      "kube-proxy" => AddOnType::KubeProxy,
      "vpc-cni" => AddOnType::VpcCni,
      "eks-pod-identity-agent" => AddOnType::EksPodIdentityAgent,
      "aws-ebs-csi-driver" => AddOnType::AwsEbsCsiDriver,
      "aws-efs-csi-driver" => AddOnType::AwsEfsCsiDriver,
      "aws-mountpoint-s3-csi-driver" => AddOnType::AwsMountpointS3CsiDriver,
      "snapshot-controller" => AddOnType::SnapshotController,
      "adot" => AddOnType::Adot,
      "aws-guardduty-agent" => AddOnType::AwsGuarddutyAgent,
      "amazon-cloudwatch-observability" => AddOnType::AmazonCloudwatchObservability,
      _ => AddOnType::CoreDns,
    }
  }
}
