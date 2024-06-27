module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = "ex"
  cluster_version = "1.30"

  cluster_addons = {
    coredns =  {}
    kube-proxy =  {}
    vpc-cni =  {}
    eks-pod-identity-agent =  {}
    aws-ebs-csi-driver = {
      service_account_role_arn = module.aws_ebs_csi_driver_irsa.iam_role_arn
    }
    aws-efs-csi-driver = {
      service_account_role_arn = module.aws_efs_csi_driver_irsa.iam_role_arn
    }
    aws-mountpoint-s3-csi-driver = {
      service_account_role_arn = module.aws_mountpoint_s3_csi_driver_irsa.iam_role_arn
    }
    snapshot-controller =  {}
    adot =  {}
    aws-guardduty-agent =  {}
    amazon-cloudwatch-observability = {
      service_account_role_arn = module.amazon_cloudwatch_observability_irsa.iam_role_arn
    }
  }

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    # This node group is for core addons such as CoreDNS
    default = {
      ami_type       = "AL2023_x86_64_STANDARD"
      instance_types = [
        "c5.18xlarge",
        "c5.24xlarge",
        "c5.2xlarge",
        "c5.4xlarge",
        "c5.9xlarge",
        "c5.large",
        "c5.xlarge",
      ]

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

module "aws_ebs_csi_driver_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.39"

  role_name             = "aws-ebs-csi-driver"
  attach_ebs_csi_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:ebs-csi-controller-sa"]
    }
  }

  tags = module.tags.tags
}

module "aws_efs_csi_driver_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.39"

  role_name             = "aws-efs-csi-driver"
  attach_efs_csi_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:efs-csi-controller-sa"]
    }
  }

  tags = module.tags.tags
}

module "aws_mountpoint_s3_csi_driver_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.39"

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

  tags = module.tags.tags
}

module "amazon_cloudwatch_observability_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.39"

  role_name                              = "amazon-cloudwatch-observability"
  attach_cloudwatch_observability_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["amazon-cloudwatch:cloudwatch-agent"]
    }
  }

  tags = module.tags.tags
}

module "tags" {
  source  = "clowdhaus/tags/aws"
  version = "~> 1.1"

  application = "cookiecluster"
  environment = "nonprod"
  repository  = "github.com/clowdhaus/cookiecluster"
}
