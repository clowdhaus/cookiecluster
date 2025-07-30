################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 21.0"

  name               = "bottlerocket-arm64"
  kubernetes_version = "1.33"

  addons = {
    coredns = {}
    eks-node-monitoring-agent = {}
    eks-pod-identity-agent = {
      before_compute = true
    }
    kube-proxy = {}
    vpc-cni = {
      before_compute = true
    }
  }

  vpc_id                   = data.aws_vpc.this.id
  control_plane_subnet_ids = data.aws_subnets.control_plane.ids
  subnet_ids               = data.aws_subnets.data_plane.ids

  eks_managed_node_groups = {
    default = {
      ami_type = "BOTTLEROCKET_ARM_64"
      instance_types = [
        "m7g.xlarge",
        "m6g.xlarge",
      ]
  
      min_size     = 2
      max_size     = 3
      desired_size = 2

      node_repair_config = {
        enabled = true
      }
    }
  }

  tags = module.tags.tags
}
