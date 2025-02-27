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
  EksNodeMonitoringAgent,
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
    AddOnType::AwsEbsCsiDriver
    | AddOnType::AwsEfsCsiDriver
    | AddOnType::AwsMountpointS3CsiDriver
    | AddOnType::AmazonCloudwatchObservability => {
      let under_name = add_on_type.to_string().replace('-', "_");
      let service_account_name = match add_on_type {
        AddOnType::AwsEbsCsiDriver => Some("ebs-csi-controller-sa".to_string()),
        AddOnType::AwsEfsCsiDriver => Some("efs-csi-controller-sa".to_string()),
        AddOnType::AwsMountpointS3CsiDriver => Some("s3-csi-driver-sa".to_string()),
        AddOnType::AmazonCloudwatchObservability => Some("cloudwatch-agent".to_string()),
        _ => None,
      };
      AddOn {
        name: add_on_type.to_string(),
        under_name: under_name.to_string(),
        configuration: AddOnConfiguration {
          pod_identity_role_arn: Some(format!("module.{under_name}_pod_identity.iam_role_arn")),
          pod_identity_service_account: service_account_name,
        },
      }
    }
    _ => AddOn {
      name: add_on_type.to_string(),
      under_name: add_on_type.to_string().replace('-', "_"),
      configuration: AddOnConfiguration {
        pod_identity_role_arn: None,
        pod_identity_service_account: None,
      },
    },
  };

  Ok(add_on)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AddOnConfiguration {
  pub pod_identity_role_arn: Option<String>,
  pub pod_identity_service_account: Option<String>,
}

impl std::fmt::Display for AddOnType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AddOnType::CoreDns => write!(f, "coredns"),
      AddOnType::KubeProxy => write!(f, "kube-proxy"),
      AddOnType::VpcCni => write!(f, "vpc-cni"),
      AddOnType::EksNodeMonitoringAgent => write!(f, "eks-node-monitoring-agent"),
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
      "eks-node-monitoring-agent" => AddOnType::EksNodeMonitoringAgent,
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

#[cfg(test)]
mod tests {
  use rstest::*;

  use super::*;

  #[rstest]
  #[case(AddOnType::CoreDns, None)]
  #[case(AddOnType::KubeProxy, None)]
  #[case(AddOnType::VpcCni, None)]
  #[case(AddOnType::EksNodeMonitoringAgent, None)]
  #[case(AddOnType::EksPodIdentityAgent, None)]
  #[case(AddOnType::AwsEbsCsiDriver, Some("module.aws_ebs_csi_driver_pod_identity.iam_role_arn".to_string()))]
  #[case(AddOnType::AwsEfsCsiDriver, Some("module.aws_efs_csi_driver_pod_identity.iam_role_arn".to_string()))]
  #[case(AddOnType::AwsMountpointS3CsiDriver, Some("module.aws_mountpoint_s3_csi_driver_pod_identity.iam_role_arn".to_string()))]
  #[case(AddOnType::SnapshotController, None)]
  #[case(AddOnType::Adot, None)]
  #[case(AddOnType::AwsGuarddutyAgent, None)]
  #[case(AddOnType::AmazonCloudwatchObservability, Some("module.amazon_cloudwatch_observability_pod_identity.iam_role_arn".to_string()))]
  fn test_get_add_on_configuration(#[case] aot: AddOnType, #[case] expected: Option<String>) {
    let add_on = get_add_on_configuration(&aot.to_string()).unwrap();
    assert_eq!(add_on.configuration.pod_identity_role_arn, expected);
  }
}
