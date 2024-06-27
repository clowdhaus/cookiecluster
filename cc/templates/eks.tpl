module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = "{{ inputs.cluster_name }}"
  cluster_version = "{{ inputs.cluster_version }}"
{{ #if inputs.cluster_endpoint_public_access }}

  # To facilitate easier interaction for demonstration purposes
  cluster_endpoint_public_access = true
{{ /if }}
{{ #if inputs.enable_cluster_creator_admin_permissions }}

  # Gives Terraform identity admin access to cluster which will
  # allow deploying resources into the cluster
  enable_cluster_creator_admin_permissions = true
{{ /if }}

  cluster_addons = {
  {{ #each add_ons as |a| }}
    {{ a.name }} = {{ #if a.configuration.service_account_role_arn }}{
      service_account_role_arn = {{ a.configuration.service_account_role_arn }}
    }
    {{ ~else }} {}{{ /if }}
  {{ /each}}
  }

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    # This node group is for core addons such as CoreDNS
    default = {
      ami_type       = "{{ inputs.ami_type }}"
      instance_types = ["m5.xlarge"]

      min_size     = 1
      max_size     = 3
      desired_size = 2
    }

    gpu = {
      ami_type       = "AL2_x86_64_GPU"
      instance_types = ["g5.8xlarge"]

      min_size     = 1
      max_size     = 1
      desired_size = 1

      # Default AMI has only 8GB of storage
      block_device_mappings = {
        xvda = {
          device_name = "/dev/xvda"
          ebs = {
            volume_size           = 256
            volume_type           = "gp3"
            delete_on_termination = true
          }
        }
      }

      taints = {
        # Ensure only GPU workloads are scheduled on this node group
        gpu = {
          key    = "nvidia.com/gpu"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      }
    }
  }

  tags = module.tags.tags
}
{{ #each add_ons as |a| }}
{{ #if a.configuration.service_account_role_arn }}

module "{{ a.under_name }}_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.39"

  {{ #if (isEqual a.name "aws-ebs-csi-driver") }}
  role_name             = "aws-ebs-csi-driver"
  attach_ebs_csi_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:ebs-csi-controller-sa"]
    }
  }
  {{ /if }}
  {{ #if (isEqual a.name "aws-efs-csi-driver") }}
  role_name             = "aws-efs-csi-driver"
  attach_efs_csi_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:efs-csi-controller-sa"]
    }
  }
  {{ /if }}
  {{ #if (isEqual a.name "aws-mountpoint-s3-csi-driver") }}
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
  {{ #if (isEqual a.name "amazon-cloudwatch-observability") }}
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

module "tags" {
  source  = "clowdhaus/tags/aws"
  version = "~> 1.1"

  application = "cookiecluster"
  environment = "nonprod"
  repository  = "github.com/clowdhaus/cookiecluster"
}
