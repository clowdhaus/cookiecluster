{{ #if (eq inputs.accelerator "Nvidia") }}
gpu = {
  ami_type       = "{{ inputs.ami_type }}"
  instance_types = instance_types = [
  {{ #each instance_types }}
  "{{ this }}",
  {{ /each }}
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

  {{ #if inputs.enable_efa}}

  # Add security group rules on the node group security group to
  # allow EFA traffic
  enable_efa_support = true
  {{ /if }}

  labels = {
    {{ #if inputs.enable_efa}}"vpc.amazonaws.com/efa.present" = "true"{{ /if }}
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

  subnet_ids = [element(module.vpc.private_subnets, 0)]
  capacity_reservation_specification = {
    capacity_reservation_target = {
      capacity_reservation_resource_group_arn = aws_resourcegroups_group.odcr.arn
    }
  }
}

################################################################################
# Resource Group
################################################################################

resource "aws_resourcegroups_group" "odcr" {
  name        = "${local.name}-p5-odcr"
  description = "P5 instance on-demand capacity reservations"

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
  count = length(var.capacity_reservation_arns)

  group_arn    = aws_resourcegroups_group.odcr.arn
  resource_arn = element(var.capacity_reservation_arns, count.index)
}
