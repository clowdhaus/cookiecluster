{{ #each add_ons as |a| }}
{{ #if a.configuration.service_account_role_arn }}

################################################################################
# Add-On IAM Role(s) for Service Account(s)
################################################################################

module "{{ a.under_name }}_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.39"

  {{ #if (eq a.name "aws-ebs-csi-driver") }}
  role_name             = "aws-ebs-csi-driver"
  attach_ebs_csi_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:ebs-csi-controller-sa"]
    }
  }
  {{ /if }}
  {{ #if (eq a.name "aws-efs-csi-driver") }}
  role_name             = "aws-efs-csi-driver"
  attach_efs_csi_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:efs-csi-controller-sa"]
    }
  }
  {{ /if }}
  {{ #if (eq a.name "aws-mountpoint-s3-csi-driver") }}
  role_name                       = "aws-mountpoint-s3-csi-driver"
  attach_mountpoint_s3_csi_policy = true
  # TODO - update with your respective S3 bucket ARN(s) and path(s)
  mountpoint_s3_csi_bucket_arns   = ["arn:aws:s3:::mountpoint-s3-csi-bucket"]
  mountpoint_s3_csi_path_arns     = ["arn:aws:s3:::mountpoint-s3-csi-bucket/example/*"]

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:s3-csi-driver-sa"]
    }
  }
  {{ /if }}
  {{ #if (eq a.name "amazon-cloudwatch-observability") }}
  role_name                              = "amazon-cloudwatch-observability"
  attach_cloudwatch_observability_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["amazon-cloudwatch:cloudwatch-agent"]
    }
  }
  {{ /if }}

  tags = module.tags.tags
}
{{ /if }}
{{ /each }}
