################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.37"

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
  {{ #if (and inputs.enable_efa (not inputs.enable_auto_mode)) }}

  # Add security group rules on the node group security group to
  # allow EFA traffic
  enable_efa_support = true
  {{ /if }}

  vpc_id                   = data.aws_vpc.this.id
  control_plane_subnet_ids = data.aws_subnets.control_plane.ids
  subnet_ids               = data.aws_subnets.data_plane.ids
  {{ #unless inputs.enable_auto_mode }}

  eks_managed_node_group_defaults = {
    node_repair_config = {
      enabled = true
    }
  }

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
