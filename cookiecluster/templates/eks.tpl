{{ #if inputs.enable_accelerator }}
/*
  {{ #if inputs.enable_nvidia_gpus }}
    {{ #if (eq inputs.ami_type "AL2023_x86_64_NVIDIA") }}
## NVIDIA K8s Device Plugin

The NVIDIA K8s device plugin, https://github.com/NVIDIA/k8s-device-plugin, will need to
be installed in the cluster in order to mount and utilize the GPUs in your pods.

By default, the NVIDIA K8s device values already contain a toleration that matches the taint applied
to the node group below, as well as a nodeAffinity selector for `nvidia.com/gpu.present` that matches the label
applied to the node group.
    {{ else }}
## NVIDIA K8s Device Plugin

The "{{ inputs.ami_type }}" AMI type comes with the NVIDIA K8s device plugin pre-installed on the AMI.
    {{ /if }}
  {{ /if }}
  {{ #if inputs.enable_neuron_devices }}
## Neuron K8s Device Plugin

The Neuron K8s device plugin, https://awsdocs-neuron.readthedocs-hosted.com/en/latest/containers/tutorials/k8s-setup.html,
will need to  be installed in the cluster in order to mount and utilize the Neuron devices
in your pods. Add the following node selector and toleration to your device plugin Daemonset
values to ensure the device plugin runs on nodes that have Neuron devices present
(as identified via the MNG label and taint provided below):

```yaml
nodeSelector:
  vpc.amazonaws.com/efa.present: 'true'
tolerations:
  - key: aws.amazon.com/neuron
    operator: Exists
    effect: NoSchedule
```

The Neuron K8s device plugin Helm chart support can be tracked in the following two GitHub issues:
- https://github.com/aws/eks-charts/issues/1068
- https://github.com/aws-neuron/aws-neuron-sdk/issues/707
  {{ /if }}
  {{ #if inputs.enable_efa }}

## EFA K8s Device Plugin

The EFA K8s device plugin, https://github.com/aws/eks-charts/tree/master/stable/aws-efa-k8s-device-plugin,
will need to be installed in the cluster in order to mount and utilize the EFA devices
in your pods. Add the following node selector and toleration to your device plugin Helm chart
values to ensure the device plugin runs on nodes that have EFA devices present
(as identified via the MNG label and taint provided below):

```yaml
nodeSelector:
  vpc.amazonaws.com/efa.present: 'true'
tolerations:
{{ #if inputs.enable_neuron_devices }}
  - key: aws.amazon.com/neuron
    operator: Exists
    effect: NoSchedule
{{ else }}
  - key: nvidia.com/gpu
    operator: Exists
    effect: NoSchedule
{{ /if }}
```
  {{ /if }}
*/
{{ /if }}
################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.34"

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
  {{ #if inputs.enable_auto_mode }}

  cluster_compute_config = {
    enabled    = true
    node_pools = ["general-purpose", "system"]
  }
  {{ else }}
  {{ #if inputs.enable_add_ons}}

  # These will become the default in the next major version of the module
  bootstrap_self_managed_addons   = false
  enable_irsa                     = false
  enable_security_groups_for_pods = false

  cluster_addons = {
  {{ #each inputs.add_ons as |a| }}
    {{ a.name }} = {{ #if a.configuration.pod_identity_role_arn }}{
      pod_identity_role_arn = [{
        role_arn        = {{ a.configuration.pod_identity_role_arn }}
        service_account = "{{ a.configuration.pod_identity_service_account }}"
      }]
      {{ #if a.configuration.before_compute }}
      before_compute = true
      {{ /if }}
    }
    {{ else if a.configuration.before_compute }}{
      before_compute = true
    }
    {{ ~else }}{}{{ /if }}
  {{ /each}}
  }
  {{ /if }}
  {{ /if }}
  {{ #if inputs.enable_efa }}

  # Add security group rules on the node group security group to
  # allow EFA traffic
  enable_efa_support = true
  {{ /if }}

  vpc_id                   = data.aws_vpc.this.id
  control_plane_subnet_ids = data.aws_subnets.control_plane.ids
  subnet_ids               = data.aws_subnets.data_plane.ids
  {{ #unless inputs.enable_auto_mode }}

  {{> tpl_node_groups }}
  {{ /unless }}

  {{ #if inputs.enable_karpenter }}
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
{{ #if (and inputs.enable_odcr (not inputs.enable_karpenter)) }}

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
{{ #if inputs.enable_pod_identity }}

{{> tpl_pod_identity }}
{{ /if }}
