################################################################################
# EKS Cluster
################################################################################

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 21.0"

  name               = "{{ inputs.name }}"
  kubernetes_version = "{{ inputs.kubernetes_version }}"
  {{ #if inputs.endpoint_public_access }}

  # To facilitate easier interaction for demonstration purposes
  endpoint_public_access = true
  {{ /if }}
  {{ #if inputs.enable_cluster_creator_admin_permissions }}

  # Gives Terraform identity admin access to cluster which will
  # allow deploying resources into the cluster
  enable_cluster_creator_admin_permissions = true
  {{ /if }}
  {{ #if inputs.enable_auto_mode }}

  compute_config = {
    enabled    = true
    node_pools = ["general-purpose", "system"]
  }
  {{ else }}
  {{ #if inputs.enable_add_ons}}

  addons = {
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
    "karpenter.sh/discovery" = "{{ inputs.name }}"
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
  name        = "{{ inputs.name }}-odcr"
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
