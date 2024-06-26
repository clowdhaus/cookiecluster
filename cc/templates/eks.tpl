module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = "{{ cluster_name }}"
  cluster_version = "{{ cluster_version }}"

  {{ #if cluster_endpoint_public_access }}
  # To facilitate easier interaction for demonstration purposes
  cluster_endpoint_public_access = true
  {{ /if }}

  {{ #if enable_cluster_creator_admin_permissions }}
  # Gives Terraform identity admin access to cluster which will
  # allow deploying resources into the cluster
  enable_cluster_creator_admin_permissions = true
  {{ /if }}

  cluster_addons = {
  {{#each add_ons }}
    {{ this }} = {}
  {{/each}}
  }

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    # This node group is for core addons such as CoreDNS
    default = {
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

  tags = {
    Environment = "test"
  }
}
