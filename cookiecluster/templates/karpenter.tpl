{{ #if inputs.enable_karpenter }}
################################################################################
# Controller & Node IAM roles, SQS Queue, Eventbridge Rules
################################################################################

module "karpenter" {
  source  = "terraform-aws-modules/eks/aws//modules/karpenter"
  version = "~> 20.34"

  cluster_name = module.eks.cluster_name

  # Enable permissions for Karpenter v1.0+
  enable_v1_permissions = true
  namespace             = "kube-system"

  # Name needs to match role name passed to the EC2NodeClass
  node_iam_role_use_name_prefix   = false
  node_iam_role_name              = "{{ inputs.cluster_name }}-karpenter-node"
  create_pod_identity_association = true

  tags = module.tags.tags
}

################################################################################
# Karpenter Helm chart
################################################################################

provider "aws" {
  region = "us-east-1"
  alias  = "virginia"
}

data "aws_ecrpublic_authorization_token" "token" {
  provider = aws.virginia
}

resource "helm_release" "karpenter" {
  namespace           = "kube-system"
  name                = "karpenter"
  repository          = "oci://public.ecr.aws/karpenter"
  repository_username = data.aws_ecrpublic_authorization_token.token.user_name
  repository_password = data.aws_ecrpublic_authorization_token.token.password
  chart               = "karpenter"
  version             = "1.3.2"
  wait                = false

  values = [
    <<-EOT
      dnsPolicy: Default
      serviceAccount:
        name: ${module.karpenter.service_account}
      settings:
        clusterName: ${module.eks.cluster_name}
        clusterEndpoint: ${module.eks.cluster_endpoint}
        interruptionQueue: ${module.karpenter.queue_name}
    EOT
  ]
}
{{ /if }}
