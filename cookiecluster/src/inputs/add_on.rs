use std::{collections::BTreeMap, sync::LazyLock};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use super::compute;

#[derive(Clone, Debug, Display, EnumString, EnumIter, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum AddOnType {
  Adot,
  AmazonCloudwatchObservability,
  AwsEbsCsiDriver,
  AwsEfsCsiDriver,
  AwsGuarddutyAgent,
  AwsMountpointS3CsiDriver,
  #[strum(serialize = "coredns")]
  #[serde(rename = "coredns")]
  CoreDns,
  EksNodeMonitoringAgent,
  EksPodIdentityAgent,
  KubeProxy,
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
  pub auto_mode: bool,
  pub default: bool,
  pub name: String,
  pub configuration: Option<AddOnConfiguration>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddOnConfiguration {
  pub pod_identity_role_arn: Option<String>,
  pub pod_identity_service_account: Option<String>,
  pub before_compute: bool,
}

static ADD_ONS: LazyLock<BTreeMap<AddOnType, AddOn>> = LazyLock::new(|| {
  BTreeMap::from([
    (
      AddOnType::Adot,
      AddOn {
        auto_mode: true,
        default: false,
        name: AddOnType::Adot.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::AmazonCloudwatchObservability,
      AddOn {
        auto_mode: true,
        default: false,
        name: AddOnType::AmazonCloudwatchObservability.to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: Some("module.amazon_cloudwatch_observability_pod_identity.iam_role_arn".to_string()),
          pod_identity_service_account: Some("cloudwatch-agent".to_string()),
          before_compute: false,
        }),
      },
    ),
    (
      AddOnType::AwsEbsCsiDriver,
      AddOn {
        auto_mode: false,
        default: false,
        name: AddOnType::AwsEbsCsiDriver.to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: Some("module.aws_ebs_csi_driver_pod_identity.iam_role_arn".to_string()),
          pod_identity_service_account: Some("ebs-csi-controller-sa".to_string()),
          before_compute: false,
        }),
      },
    ),
    (
      AddOnType::AwsEfsCsiDriver,
      AddOn {
        auto_mode: true,
        default: false,
        name: AddOnType::AwsEfsCsiDriver.to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: Some("module.aws_efs_csi_driver_pod_identity.iam_role_arn".to_string()),
          pod_identity_service_account: Some("efs-csi-controller-sa".to_string()),
          before_compute: false,
        }),
      },
    ),
    (
      AddOnType::AwsGuarddutyAgent,
      AddOn {
        auto_mode: false,
        default: false,
        name: AddOnType::AwsGuarddutyAgent.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::AwsMountpointS3CsiDriver,
      AddOn {
        auto_mode: true,
        default: false,
        name: AddOnType::AwsMountpointS3CsiDriver.to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: Some("module.aws_mountpoint_s3_csi_driver_pod_identity.iam_role_arn".to_string()),
          pod_identity_service_account: Some("s3-csi-driver-sa".to_string()),
          before_compute: false,
        }),
      },
    ),
    (
      AddOnType::CoreDns,
      AddOn {
        auto_mode: false,
        default: true,
        name: AddOnType::CoreDns.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::EksNodeMonitoringAgent,
      AddOn {
        auto_mode: false,
        default: true,
        name: AddOnType::EksNodeMonitoringAgent.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::EksPodIdentityAgent,
      AddOn {
        auto_mode: false,
        default: true,
        name: AddOnType::EksPodIdentityAgent.to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: None,
          pod_identity_service_account: None,
          before_compute: true,
        }),
      },
    ),
    (
      AddOnType::KubeProxy,
      AddOn {
        auto_mode: false,
        default: true,
        name: AddOnType::KubeProxy.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::SnapshotController,
      AddOn {
        auto_mode: false,
        default: false,
        name: AddOnType::SnapshotController.to_string(),
        configuration: None,
      },
    ),
    (
      AddOnType::VpcCni,
      AddOn {
        auto_mode: false,
        default: true,
        name: AddOnType::VpcCni.to_string(),
        configuration: Some(AddOnConfiguration {
          pod_identity_role_arn: None,
          pod_identity_service_account: None,
          before_compute: true,
        }),
      },
    ),
  ])
});

/// Get all available add-ons
#[inline]
#[cfg(not(tarpaulin_include))]
pub fn _get_all_add_on_types() -> Vec<AddOnType> {
  ADD_ONS.keys().cloned().collect()
}

#[inline]
#[cfg(not(tarpaulin_include))]
pub fn get_add_on_names(scaling_type: &compute::ScalingType) -> Vec<&'static str> {
  ADD_ONS
    .iter()
    .filter(|(_, v)| match scaling_type {
      compute::ScalingType::AutoMode => v.auto_mode,
      _ => true,
    })
    .map(|(_, v)| v.name.as_str())
    .collect::<Vec<_>>()
}

#[inline]
#[cfg(not(tarpaulin_include))]
pub fn get_default_add_on_types() -> Vec<AddOnType> {
  ADD_ONS
    .iter()
    .filter(|(_, v)| v.default)
    .map(|(k, _v)| k.clone())
    .collect::<Vec<_>>()
}

#[inline]
#[cfg(not(tarpaulin_include))]
pub fn get_default_add_on_flags() -> Vec<bool> {
  ADD_ONS.values().map(|v| v.default).collect::<Vec<_>>()
}

#[cfg(not(tarpaulin_include))]
pub fn get_add_on(add_on_type: &AddOnType) -> Result<AddOn> {
  let config = ADD_ONS
    .get(add_on_type)
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
    let add_on = get_add_on(&aot).unwrap();
    let result = match add_on.configuration {
      Some(c) => c.pod_identity_role_arn,
      None => None,
    };
    assert_eq!(result, expected);
  }
}
