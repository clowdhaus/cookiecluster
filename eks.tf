# data "aws_subnets" "control_plane" {
#   filter {
#     name   = "vpc-id"
#     values = [var.vpc_id]
#   }
# }

# data "aws_subnets" "data_plane" {
#   filter {
#     name   = "vpc-id"
#     values = [var.vpc_id]
#   }
# }

################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = "example"
  cluster_version = "1.30"

  cluster_addons = {
    coredns =  {}
    kube-proxy =  {}
    vpc-cni =  {}
    eks-pod-identity-agent =  {}
  }

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    # This node group is for core addons such as CoreDNS
    default = {
      ami_type = "AL2023_x86_64_STANDARD"
      instance_types = [
        "c5.12xlarge",
      ]

      min_size     = 1
      max_size     = 3
      desired_size = 2
    }
  }

  tags = module.tags.tags
}
################################################################################
# Tags - Replace with your own tags implementation
################################################################################

module "tags" {
  source  = "clowdhaus/tags/aws"
  version = "~> 1.1"

  application = "cookiecluster"
  environment = "nonprod"
  repository  = "github.com/clowdhaus/cookiecluster"
}
