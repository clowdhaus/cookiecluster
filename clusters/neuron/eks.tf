################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 21.0"

  name               = "neuron"
  kubernetes_version = "1.33"

  addons = {
    coredns                   = {}
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
    neuron = {
      ami_type = "AL2023_x86_64_NEURON"
      instance_types = [
        "inf2.xlarge",
      ]

      min_size     = 0
      max_size     = 5
      desired_size = 2

      # Increase root EBS volume size (default is 8Gb)
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

      labels = {
        "aws.amazon.com/neuron.present" = "true"
      }

      taints = {
        # Ensure only Neuron workloads are scheduled on this node group
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
