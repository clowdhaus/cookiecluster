################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 21.0"

  name               = "example"
  kubernetes_version = "1.33"

  addons = {
    coredns = {}
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
      ami_type = "AL2023_x86_64_STANDARD"
      instance_types = [
        "m7a.xlarge",
        "m7i.xlarge",
      ]
  
      min_size     = 2
      max_size     = 3
      desired_size = 2

      # Increase root EBS volume
      block_device_mappings = {
        xvda = {
          device_name = "/dev/xvda"
          ebs = {
            volume_size = 24
          }
        }
      }

      node_repair_config = {
        enabled = true
      }
    }
  }

  tags = module.tags.tags
}
