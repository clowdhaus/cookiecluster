terraform {
  required_version = ">= 1.3.2"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.83"
    }
    {{ #if inputs.enable_helm }}
    helm = {
      source  = "hashicorp/helm"
      version = ">= 2.16"
    }
    {{ /if }}
  }
}
{{ #if inputs.enable_helm }}

provider "helm" {
  kubernetes {
    host                   = module.eks.cluster_endpoint
    cluster_ca_certificate = base64decode(module.eks.cluster_certificate_authority_data)

    exec {
      api_version = "client.authentication.k8s.io/v1beta1"
      command     = "aws"
      # This requires the awscli to be installed locally where Terraform is executed
      args = ["eks", "get-token", "--cluster-name", module.eks.cluster_name]
    }
  }
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
{{ #if inputs.enable_compute_reservation }}

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
