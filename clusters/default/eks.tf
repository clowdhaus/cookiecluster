################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 21.0"

  name               = "example"
  kubernetes_version = "1.33"

  compute_config = {
    enabled    = true
    node_pools = ["general-purpose", "system"]
  }

  vpc_id                   = data.aws_vpc.this.id
  control_plane_subnet_ids = data.aws_subnets.control_plane.ids
  subnet_ids               = data.aws_subnets.data_plane.ids

  tags = module.tags.tags
}
