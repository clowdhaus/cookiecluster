use std::{collections::BTreeMap, sync::LazyLock};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(
  Debug, EnumIter, Display, EnumString, IntoStaticStr, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[strum(serialize_all = "kebab-case")]
pub enum AddOnType {
  Adot,
  AmazonCloudwatchObservability,
  AwsEbsCsiDriver,
  AwsEfsCsiDriver,
  AwsGuarddutyAgent,
  AwsMountpointS3CsiDriver,
  #[strum(serialize = "coredns")]
  CoreDns,
  KubeProxy,
  EksNodeMonitoringAgent,
  EksPodIdentityAgent,
  SnapshotController,
  VpcCni,
}

impl AddOnType {
  #[cfg(not(tarpaulin_include))]
  pub fn from_idx(idx: usize) -> AddOnType {
    AddOnType::iter().nth(idx).unwrap()
  }
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
        name: AddOnType::Adot.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::AmazonCloudwatchObservability,
      AddOn {
        default: false,
        name: AddOnType::AmazonCloudwatchObservability.to_string(),
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
        name: AddOnType::AwsEbsCsiDriver.to_string(),
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
        name: AddOnType::AwsEfsCsiDriver.to_string(),
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
        name: AddOnType::AwsGuarddutyAgent.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::AwsMountpointS3CsiDriver,
      AddOn {
        default: false,
        name: AddOnType::AwsMountpointS3CsiDriver.to_string(),
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
        name: AddOnType::CoreDns.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::KubeProxy,
      AddOn {
        default: true,
        name: AddOnType::KubeProxy.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::EksNodeMonitoringAgent,
      AddOn {
        default: true,
        name: AddOnType::EksNodeMonitoringAgent.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::EksPodIdentityAgent,
      AddOn {
        default: true,
        name: AddOnType::EksPodIdentityAgent.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::SnapshotController,
      AddOn {
        default: false,
        name: AddOnType::SnapshotController.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::VpcCni,
      AddOn {
        default: true,
        name: AddOnType::VpcCni.to_string(),
        configuration: None,
      },
    ),
  ])
});

/// Get all available add-ons
#[inline]
#[cfg(not(tarpaulin_include))]
pub fn _get_all_add_ons() -> Vec<AddOn> {
  ADD_ONS.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
}

#[inline]
#[cfg(not(tarpaulin_include))]
pub fn get_add_on_names() -> Vec<&'static str> {
  AddOnType::iter().map(|aot| aot.into()).collect()
}

#[inline]
#[cfg(not(tarpaulin_include))]
pub fn get_default_add_ons() -> Vec<AddOn> {
  ADD_ONS
    .iter()
    .filter(|(_, v)| v.default)
    .map(|(_, v)| v.clone())
    .collect::<Vec<_>>()
}

#[inline]
#[cfg(not(tarpaulin_include))]
pub fn get_default_add_on_flags() -> Vec<bool> {
  ADD_ONS.iter().map(|(_, v)| v.default).collect::<Vec<_>>()
}

#[cfg(not(tarpaulin_include))]
pub fn get_add_on(add_on_type: AddOnType) -> Result<AddOn> {
  let config = ADD_ONS
    .get(&add_on_type)
    .ok_or_else(|| anyhow::anyhow!("Add-on not found"))?;
  Ok(config.clone())
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
