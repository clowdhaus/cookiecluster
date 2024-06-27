neuron = {
      ami_type       = "{{ inputs.ami_type }}"
      instance_types = [
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