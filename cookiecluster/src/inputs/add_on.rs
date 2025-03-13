use std::{collections::BTreeMap, fmt, sync::LazyLock};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddOnType {
  Adot,
  AmazonCloudwatchObservability,
  AwsEbsCsiDriver,
  AwsEfsCsiDriver,
  AwsGuarddutyAgent,
  AwsMountpointS3CsiDriver,
  CoreDns,
  KubeProxy,
  EksNodeMonitoringAgent,
  EksPodIdentityAgent,
  SnapshotController,
  VpcCni,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddOn {
  pub default: bool,
  pub name: String,
  pub configuration: Option<AddOnConfiguration>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddOnConfiguration {
  pub pod_identity_role_arn: Option<String>,
  pub pod_identity_service_account: Option<String>,
}

static ADD_ONS: LazyLock<BTreeMap<AddOnType, AddOn>> = LazyLock::new(|| {
  BTreeMap::from([
    (
      AddOnType::Adot,
      AddOn {
        default: false,
        name: "adot".to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::AmazonCloudwatchObservability,
      AddOn {
        default: false,
        name: "amazon-cloudwatch-observability".to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: Some("module.amazon_cloudwatch_observability_pod_identity.iam_role_arn".to_string()),
          pod_identity_service_account: Some("cloudwatch-agent".to_string()),
        }),
      },
    ),
    (
      AddOnType::AwsEbsCsiDriver,
      AddOn {
        default: false,
        name: "aws-ebs-csi-driver".to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: Some("module.aws_ebs_csi_driver_pod_identity.iam_role_arn".to_string()),
          pod_identity_service_account: Some("ebs-csi-controller-sa".to_string()),
        }),
      },
    ),
    (
      AddOnType::AwsEfsCsiDriver,
      AddOn {
        default: false,
        name: "aws-efs-csi-driver".to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: Some("module.aws_efs_csi_driver_pod_identity.iam_role_arn".to_string()),
          pod_identity_service_account: Some("efs-csi-controller-sa".to_string()),
        }),
      },
    ),
    (
      AddOnType::AwsGuarddutyAgent,
      AddOn {
        default: false,
        name: "aws-guardduty-agent".to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::AwsMountpointS3CsiDriver,
      AddOn {
        default: false,
        name: "aws-mountpoint-s3-csi-driver".to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: Some("module.aws_mountpoint_s3_csi_driver_pod_identity.iam_role_arn".to_string()),
          pod_identity_service_account: Some("s3-csi-driver-sa".to_string()),
        }),
      },
    ),
    (
      AddOnType::CoreDns,
      AddOn {
        default: true,
        name: "coredns".to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::KubeProxy,
      AddOn {
        default: true,
        name: "kube-proxy".to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::EksNodeMonitoringAgent,
      AddOn {
        default: true,
        name: "eks-node-monitoring-agent".to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::EksPodIdentityAgent,
      AddOn {
        default: true,
        name: "eks-pod-identity-agent".to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::SnapshotController,
      AddOn {
        default: false,
        name: "snapshot-controller".to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::VpcCni,
      AddOn {
        default: true,
        name: "vpc-cni".to_string(),
        configuration: None,
      },
    ),
  ])
});

/// Get all available add-ons
#[inline]
pub fn _get_all_add_ons() -> Vec<AddOn> {
  ADD_ONS.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
}

#[inline]
pub fn get_add_on_names() -> Vec<String> {
  AddOnType::iter().map(|v| v.to_string()).collect::<Vec<_>>()
}

#[inline]
pub fn get_default_add_ons() -> Vec<AddOn> {
  ADD_ONS
    .iter()
    .filter(|(_, v)| v.default)
    .map(|(_, v)| v.clone())
    .collect::<Vec<_>>()
}

#[inline]
pub fn get_default_add_on_flags() -> Vec<bool> {
  ADD_ONS.iter().map(|(_, v)| v.default).collect::<Vec<_>>()
}

pub fn get_add_on(add_on_type: AddOnType) -> Result<AddOn> {
  let config = ADD_ONS
    .get(&add_on_type)
    .ok_or_else(|| anyhow::anyhow!("Add-on not found"))?;
  Ok(config.clone())
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
    let add_on = get_add_on(aot).unwrap();
    let result = match add_on.configuration {
      Some(c) => c.pod_identity_role_arn,
      None => None,
    };
    assert_eq!(result, expected);
  }
}
