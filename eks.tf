data "aws_vpc" "this" {
  filter {
    name   = "tag:Name"
    values = ["example"]
  }
}

data "aws_subnets" "control_plane" {
  filter {
    name   = "tag:Name"
    values = ["*-private-*"]
  }
}

data "aws_subnets" "data_plane" {
  filter {
    name   = "tag:Name"
    values = ["*-private-*"]
  }
}

data "aws_subnets" "data_plane_reservation" {
  filter {
    name   = "tag:Name"
    values = ["*-private-*"]
  }

  # Capacity reservations are restricted to a single availability zone
  filter {
    name = "availability-zone"
    values = ["us-west-2a"]
  }
}

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

  vpc_id                   = data.aws_vpc.this.id
  control_plane_subnet_ids = data.aws_subnets.control_plane.ids
  subnet_ids               = data.aws_subnets.data_plane.ids

  eks_managed_node_groups = {
    # This node group is for core addons such as CoreDNS
    default = {
      ami_type = "AL2023_x86_64_STANDARD"
      instance_types = [
        "m7a.xlarge",
        "m7i.xlarge",
      ]

      min_size     = 2
      max_size     = 3
      desired_size = 2
    }
    gpu = {
      ami_type       = "AL2_x86_64_GPU"
      instance_types = ["p5.48xlarge" ]

      min_size     = 2
      max_size     = 5
      desired_size = 2

      pre_bootstrap_user_data = <<-EOT
        #!/usr/bin/env bash

        # Mount instance store volumes in RAID-0 for Kubelet and Containerd (raid0)
        # https://github.com/awslabs/amazon-eks-ami/blob/master/doc/USER_GUIDE.md#raid-0-for-kubelet-and-containerd-raid0
        /bin/setup-local-disks raid0
      EOT

      # Expand AMI root volume
      block_device_mappings = {
        xvda = {
          device_name = "/dev/xvda"
          ebs = {
            volume_size           = 24
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
        "nvidia.com/gpu.present"        = "true"
      }

      taints = {
        # Ensure only GPU workloads are scheduled on this node group
        gpu = {
          key    = "nvidia.com/gpu"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      }

      # Capacity reservations are restricted to a single availability zone
      subnet_ids = data.aws_subnets.data_plane_reservation.ids
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
