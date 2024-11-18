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
    {{ #if (or (eq inputs.ami_type "AL2023_ARM_64_STANDARD") (eq inputs.ami_type "AL2023_x86_64_STANDARD") (eq inputs.ami_type "AL2023_x86_64_NVIDIA") (eq inputs.ami_type "AL2023_x86_64_NEURON")) }}

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
    {{ #if (or (eq inputs.ami_type "AL2023_ARM_64_STANDARD") (eq inputs.ami_type "AL2023_x86_64_STANDARD")) }}
    
    # Increase root EBS volume
    block_device_mappings = {
      xvda = {
        device_name = "/dev/xvda"
        ebs = {
          volume_size = 24
        }
      }
    }
    {{ /if }}
    {{ /if }}
  }
  {{ /if }}
  {{ #if (or (eq inputs.accelerator "Neuron") (eq inputs.accelerator "NVIDIA")) }}
  {{> tpl_node_group_accel }}
  {{ /if }}
}
