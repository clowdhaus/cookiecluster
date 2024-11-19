################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.29"

  cluster_name    = "example"
  cluster_version = "1.31"

  cluster_addons = {
    coredns = {
      configuration_values = jsonencode({
        tolerations = [
          # Allow CoreDNS to run on the same nodes as the Karpenter controller
          # for use during cluster creation when Karpenter nodes do not yet exist
          {
            key    = "karpenter.sh/controller"
            value  = "true"
            effect = "NoSchedule"
          }
        ]
      })
    }
    kube-proxy = {}
    vpc-cni = {
      pod_identity_role_arn = [{
        role_arn        = module.vpc_cni_pod_identity.iam_role_arn
        service_account = "aws-node"
      }]
    }
    eks-pod-identity-agent = {}
  }

  vpc_id                   = data.aws_vpc.this.id
  control_plane_subnet_ids = data.aws_subnets.control_plane.ids
  subnet_ids               = data.aws_subnets.data_plane.ids

  cluster_zonal_shift_config = {
    enabled = true
  }

  eks_managed_node_groups = {
    karpenter = {
      ami_type = "AL2023_x86_64_STANDARD"
      instance_types = [
        "m7a.xlarge",
        "m7i.xlarge",
      ]
  
      min_size     = 2
      max_size     = 3
      desired_size = 2
  
      labels = {
        # Used to ensure Karpenter runs on nodes that it does not manage
        "karpenter.sh/controller" = "true"
      }
  
      taints = {
        # The pods that do not tolerate this taint should run on nodes
        # created by Karpenter
        karpenter = {
          key    = "karpenter.sh/controller"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      }
    }
  }

  tags = merge(module.tags.tags, {
    # NOTE - if creating multiple security groups with this module, only tag the
    # security group that Karpenter should utilize with the following tag
    # (i.e. - at most, only one security group should have this tag in your account)
    "karpenter.sh/discovery" = example
  })
}

################################################################################
# EKS Pod Identity IAM Roles
################################################################################

module "vpc_cni_pod_identity" {
  source = "terraform-aws-modules/eks-pod-identity/aws"
  version = "~> 1.7"

  name = "vpc-cni"

  attach_aws_vpc_cni_policy = true
  aws_vpc_cni_enable_ipv4   = true

  tags = module.tags.tags
}

