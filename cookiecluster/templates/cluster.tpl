data "aws_vpc" "this" {
  filter {
    name   = "tag:Name"
    values = ["{{ inputs.vpc_name }}"]
  }
}

data "aws_subnets" "control_plane" {
  filter {
    name   = "tag:Name"
    values = ["{{ inputs.control_plane_subnet_filter }}"]
  }
}

data "aws_subnets" "data_plane" {
  filter {
    name   = "tag:Name"
    values = ["{{ inputs.data_plane_subnet_filter }}"]
  }
}
{{ #if (or (eq inputs.reservation "ODCR") (eq inputs.reservation "CBR")) }}

data "aws_subnets" "data_plane_reservation" {
  filter {
    name   = "tag:Name"
    values = ["{{ inputs.data_plane_subnet_filter }}"]
  }

  # Capacity reservations are restricted to a single availability zone
  filter {
    name = "availability-zone"
    values = ["{{ inputs.reservation_availability_zone }}"]
  }
}
{{ /if }}

################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = "{{ inputs.cluster_name }}"
  cluster_version = "{{ inputs.cluster_version }}"
  {{ #if inputs.cluster_endpoint_public_access }}

  # To facilitate easier interaction for demonstration purposes
  cluster_endpoint_public_access = true
  {{ /if }}
  {{ #if inputs.enable_cluster_creator_admin_permissions }}

  # Gives Terraform identity admin access to cluster which will
  # allow deploying resources into the cluster
  enable_cluster_creator_admin_permissions = true
  {{ /if }}
  {{ #if add_ons }}

  cluster_addons = {
  {{ #each add_ons as |a| }}
    {{ a.name }} = {{ #if a.configuration.service_account_role_arn }}{
      service_account_role_arn = {{ a.configuration.service_account_role_arn }}
    }{{ else if (and (eq a.name "coredns") (eq ../inputs.compute_scaling "karpenter")) }}{
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
    {{ ~else }}{}{{ /if }}
  {{ /each}}
  }
  {{ /if }}
  {{ #if inputs.enable_efa}}

  # Add security group rules on the node group security group to
  # allow EFA traffic
  enable_efa_support = true
  {{ /if }}

  vpc_id                   = data.aws_vpc.this.id
  control_plane_subnet_ids = data.aws_subnets.control_plane.ids
  subnet_ids               = data.aws_subnets.data_plane.ids

  eks_managed_node_groups = {
    {{ #if (or (eq inputs.accelerator "Neuron") (eq inputs.accelerator "NVIDIA")) }}
    # This node group is for core addons such as CoreDNS
    default = {
      ami_type = "{{ inputs.default_ami_type }}"
      instance_types = [
      {{ #each inputs.default_instance_types }}
        "{{ this }}",
      {{ /each }}
      ]

      min_size     = 2
      max_size     = 3
      desired_size = 2
    }
    {{ else if (eq inputs.compute_scaling "karpenter") }}
    karpenter = {
      ami_type = "{{ inputs.default_ami_type }}"
      instance_types = [
      {{ #each inputs.default_instance_types }}
        "{{ this }}",
      {{ /each }}
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
    {{ else }}
    default = {
      ami_type = "{{ inputs.ami_type }}"
      instance_types = [
      {{ #each inputs.instance_types }}
        "{{ this }}",
      {{ /each }}
      ]

      min_size     = 2
      max_size     = 3
      desired_size = 2
      {{ #if inputs.instance_storage_supported }}
      {{ #if (or (eq inputs.ami_type "AL2_ARM_64") (eq inputs.ami_type "AL2_x86_64")) }}

      pre_bootstrap_user_data = <<-EOT
        #!/usr/bin/env bash

        # Mount instance store volumes in RAID-0 for Kubelet and Containerd (raid0)
        # https://github.com/awslabs/amazon-eks-ami/blob/master/doc/USER_GUIDE.md#raid-0-for-kubelet-and-containerd-raid0
        /bin/setup-local-disks raid0
      EOT
      {{ /if }}
      {{ #if (or (eq inputs.ami_type "AL2023_ARM_64_STANDARD") (eq inputs.ami_type "AL2023_x86_64_STANDARD")) }}

      cloudinit_pre_nodeadm = [
        {
          content_type = "application/node.eks.aws"
          content      = <<-EOT
            ---
            apiVersion: node.eks.aws/v1alpha1
            kind: NodeConfig
            spec:
              instance:
                localStorage:
                  strategy: RAID0
          EOT
        }
      ]
      {{ /if }}
      {{ else }}
      {{ #if (or (eq inputs.ami_type "AL2_ARM_64") (eq inputs.ami_type "AL2_x86_64") (eq inputs.ami_type "AL2023_ARM_64_STANDARD") (eq inputs.ami_type "AL2023_x86_64_STANDARD")) }}
      # Increase root EBS volume size
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
      {{ /if }}
      {{ /if }}
    }
    {{ /if }}
    {{ #if (or (eq inputs.accelerator "Neuron") (eq inputs.accelerator "NVIDIA")) }}
    {{{ accelerated_mng }}}
    {{ /if }}
  }

  {{ #if (eq inputs.compute_scaling "karpenter") }}
  tags = merge(module.tags.tags, {
    # NOTE - if creating multiple security groups with this module, only tag the
    # security group that Karpenter should utilize with the following tag
    # (i.e. - at most, only one security group should have this tag in your account)
    "karpenter.sh/discovery" = {{ inputs.cluster_name }}
  })
  {{ else }}
  tags = module.tags.tags
  {{ /if }}
}
{{ #if (eq inputs.compute_scaling "karpenter") }}

################################################################################
# Controller & Node IAM roles, SQS Queue, Eventbridge Rules
################################################################################

module "karpenter" {
  source  = "terraform-aws-modules/eks/aws//modules/karpenter"
  version = "~> 20.0"

  cluster_name = module.eks.cluster_name

  # Name needs to match role name passed to the EC2NodeClass
  node_iam_role_use_name_prefix   = false
  node_iam_role_name              = "{{ inputs.cluster_name }}-karpenter-node"
  create_pod_identity_association = true

  tags = module.tags.tags
}
{{ /if }}
{{ #each add_ons as |a| }}
{{ #if a.configuration.service_account_role_arn }}

################################################################################
# Add-On IAM Role(s) for Service Account(s)
################################################################################

module "{{ a.under_name }}_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.39"

  {{ #if (eq a.name "aws-ebs-csi-driver") }}
  role_name             = "aws-ebs-csi-driver"
  attach_ebs_csi_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:ebs-csi-controller-sa"]
    }
  }
  {{ /if }}
  {{ #if (eq a.name "aws-efs-csi-driver") }}
  role_name             = "aws-efs-csi-driver"
  attach_efs_csi_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:efs-csi-controller-sa"]
    }
  }
  {{ /if }}
  {{ #if (eq a.name "aws-mountpoint-s3-csi-driver") }}
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
  {{ /if }}
  {{ #if (eq a.name "amazon-cloudwatch-observability") }}
  role_name                              = "amazon-cloudwatch-observability"
  attach_cloudwatch_observability_policy = true

  oidc_providers = {
    this = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["amazon-cloudwatch:cloudwatch-agent"]
    }
  }
  {{ /if }}

  tags = module.tags.tags
}
{{ /if }}
{{ /each }}
{{ #if (eq inputs.reservation "ODCR") }}

################################################################################
# Resource Group
################################################################################

resource "aws_resourcegroups_group" "odcr" {
  name        = "{{ inputs.cluster_name }}-odcr"
  description = "On-demand capacity reservations"

  configuration {
    type = "AWS::EC2::CapacityReservationPool"
  }

  configuration {
    type = "AWS::ResourceGroups::Generic"

    parameters {
      name   = "allowed-resource-types"
      values = ["AWS::EC2::CapacityReservation"]
    }
  }
}

resource "aws_resourcegroups_resource" "odcr" {
  count = length(var.on_demand_capacity_reservation_arns)

  group_arn    = aws_resourcegroups_group.odcr.arn
  resource_arn = element(var.on_demand_capacity_reservation_arns, count.index)
}
{{ /if }}

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
{{ #if (eq inputs.reservation "ODCR") }}

################################################################################
# Variables - Required input
################################################################################

variable "on_demand_capacity_reservation_arns" {
  description = "List of the on-demand capacity reservations ARNs to associate with the node group"
  type        = list(string)
  default     = []
}
{{ /if }}
