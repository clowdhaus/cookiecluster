{{ #if (eq inputs.accelerator "NVIDIA") }}
gpu = {
{{ /if }}
{{ #if (eq inputs.accelerator "Neuron") }}
neuron = {
{{ /if }}
      {{ #if (or inputs.enable_efa (eq inputs.reservation "ODCR") (eq inputs.reservation "CBR")) }}
      ami_type       = "{{ inputs.ami_type }}"
      instance_types = [{{ #each inputs.instance_types }}"{{ this }}" {{ /each }}]
      {{ else }}
      ami_type = "{{ inputs.ami_type }}"
      instance_types = [
      {{ #each inputs.instance_types }}
        "{{ this }}",
      {{ /each }}
      ]
      {{ /if }}

      min_size     = 2
      max_size     = 5
      desired_size = 2
      {{ #if inputs.instance_storage_supported }}

      pre_bootstrap_user_data = <<-EOT
        #!/usr/bin/env bash

        # Mount instance store volumes in RAID-0 for Kubelet and Containerd (raid0)
        # https://github.com/awslabs/amazon-eks-ami/blob/master/doc/USER_GUIDE.md#raid-0-for-kubelet-and-containerd-raid0
        /bin/setup-local-disks raid0
      EOT
      {{ else }}

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
      {{ /if }}
      {{ #if inputs.enable_efa}}

      # Add security group rules on the node group security group to
      # allow EFA traffic
      enable_efa_support = true
      {{ /if }}

      labels = {
        {{ #if inputs.enable_efa}}
        "vpc.amazonaws.com/efa.present" = "true"
        {{ /if }}
        {{ #if (eq inputs.accelerator "NVIDIA") }}
        "nvidia.com/gpu.present"        = "true"
        {{ /if }}
        {{ #if (eq inputs.accelerator "Neuron") }}
        "aws.amazon.com/neuron.present" = "true"
        {{ /if }}
      }

      taints = {
        {{ #if (eq inputs.accelerator "NVIDIA") }}
        # Ensure only GPU workloads are scheduled on this node group
        gpu = {
          key    = "nvidia.com/gpu"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
        {{ /if }}
        {{ #if (eq inputs.accelerator "Neuron") }}
        # Ensure only Neuron workloads are scheduled on this node group
        neuron = {
          key    = "aws.amazon.com/neuron"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
        {{ /if }}
      }
      {{ #if (or (eq inputs.reservation "ODCR") (eq inputs.reservation "CBR")) }}

      # Capacity reservations are restricted to a single availability zone
      subnet_ids = data.aws_subnets.data_plane_reservation.ids
      {{ /if }}
      {{ #if (eq inputs.reservation "ODCR") }}

      capacity_reservation_specification = {
        capacity_reservation_target = {
          capacity_reservation_resource_group_arn = aws_resourcegroups_group.odcr.arn
        }
      }
      {{ /if }}
    }