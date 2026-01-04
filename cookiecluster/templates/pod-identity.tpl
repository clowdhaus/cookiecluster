################################################################################
# EKS Pod Identity IAM Roles
################################################################################

{{ #each inputs.add_ons as |a| }}
{{ #if a.configuration.pod_identity_role_arn }}
module "{{ camel_case a.name }}_pod_identity" {
  source = "terraform-aws-modules/eks-pod-identity/aws"
  version = "~> 2.6"

  {{ #if (eq a.name "aws-ebs-csi-driver") }}
  name = "aws-ebs-csi"

  attach_aws_ebs_csi_policy = true
  {{ /if }}
  {{ #if (eq a.name "aws-efs-csi-driver") }}
  name = "aws-efs-csi"

  attach_aws_efs_csi_policy = true
  {{ /if }}
  {{ #if (eq a.name "aws-mountpoint-s3-csi-driver") }}
  name = "mountpoint-s3-csi"

  attach_mountpoint_s3_csi_policy = true
  # TODO - update with your respective S3 bucket ARN(s) and path(s)
  mountpoint_s3_csi_bucket_arns      = ["arn:aws:s3:::mountpoint-s3"]
  mountpoint_s3_csi_bucket_path_arns = ["arn:aws:s3:::mountpoint-s3/example/*"]
  {{ /if }}
  {{ #if (eq a.name "amazon-cloudwatch-observability") }}
  name = "aws-cloudwatch-observability"

  attach_aws_cloudwatch_observability_policy = true
  {{ /if }}

  tags = module.tags.tags
}

{{ /if }}
{{ /each }}
