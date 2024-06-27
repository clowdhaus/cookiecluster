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

  # Add security group rules on the node group security group to
  # allow EFA traffic
  enable_efa_support = true

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    # This node group is for core addons such as CoreDNS
    default = {
      ami_type       = "AL2_x86_64_GPU"
      instance_types = [
        "inf1.24xlarge",
      ]

      min_size     = 1
      max_size     = 3
      desired_size = 2
    }
    neuron = {
      ami_type       = "AL2_x86_64_GPU"
      instance_types = [
        "inf1.24xlarge",
      ]

      min_size     = 2
      max_size     = 5
      desired_size = 2

      pre_bootstrap_user_data = <<-EOT
        #!/usr/bin/env bash

        # Mount instance store volumes in RAID-0 for Kubelet and Containerd (raid0)
        # https://github.com/awslabs/amazon-eks-ami/blob/master/doc/USER_GUIDE.md#raid-0-for-kubelet-and-containerd-raid0
        /bin/setup-local-disks raid0
      EOT

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

      # Add security group rules on the node group security group to
      # allow EFA traffic
      enable_efa_support = true

      labels = {
        "vpc.amazonaws.com/efa.present" = "true"
        "aws.amazon.com/neuron.present" = "true"
      }

      taints = {
        # Ensure only GPU workloads are scheduled on this node group
        neuron = {
          key    = "aws.amazon.com/neuron"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      }
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
