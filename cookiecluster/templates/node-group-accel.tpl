{{ #if inputs.enable_nvidia_gpus }}
gpu = {
{{ /if }}
{{ #if inputs.enable_neuron_devices }}
neuron = {
{{ /if }}
  {{ #if (or inputs.enable_efa inputs.enable_compute_reservation) }}
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

  node_repair_config = {
    enabled = true
  }
  {{ #if inputs.instance_storage_supported }}
  {{ #if (or (eq inputs.ami_type "AL2023_x86_64_NVIDIA") (eq inputs.ami_type "AL2023_x86_64_NEURON")) }}

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
  enable_efa_only    = true
  efa_indices        = [0]
  {{ /if }}

  labels = {
    {{ #if inputs.enable_efa}}
    "vpc.amazonaws.com/efa.present" = "true"
    {{ /if }}
    {{ #if inputs.enable_nvidia_gpus }}
    "nvidia.com/gpu.present"        = "true"
    {{ /if }}
    {{ #if inputs.enable_neuron_devices }}
    "aws.amazon.com/neuron.present" = "true"
    {{ /if }}
  }

  taints = {
    {{ #if inputs.enable_nvidia_gpus }}
    # Ensure only GPU workloads are scheduled on this node group
    gpu = {
      key    = "nvidia.com/gpu"
      value  = "true"
      effect = "NO_SCHEDULE"
    }
    {{ /if }}
    {{ #if inputs.enable_neuron_devices }}
    # Ensure only Neuron workloads are scheduled on this node group
    neuron = {
      key    = "aws.amazon.com/neuron"
      value  = "true"
      effect = "NO_SCHEDULE"
    }
    {{ /if }}
  }
  {{ #if inputs.enable_compute_reservation }}

  # Capacity reservations are restricted to a single availability zone
  subnet_ids = data.aws_subnets.data_plane_reservation.ids
  {{ /if }}
  {{ #if inputs.enable_odcr }}

  capacity_reservation_specification = {
    capacity_reservation_target = {
      capacity_reservation_resource_group_arn = aws_resourcegroups_group.odcr.arn
    }
  }
  {{ else if inputs.enable_ml_cbr }}

  # ML capacity block reservation
  capacity_type = "CAPACITY_BLOCK"
  instance_market_options = {
    market_type = "capacity-block"
  }
  capacity_reservation_specification = {
    capacity_reservation_target = {
      capacity_reservation_id = var.capacity_reservation_id
    }
  }
  {{ /if }}
}
