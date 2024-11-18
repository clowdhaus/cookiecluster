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

################################################################################
# VPC & Subnet data sources
################################################################################

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
