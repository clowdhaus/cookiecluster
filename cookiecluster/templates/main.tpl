terraform {
  required_version = ">= 1.3.2"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.83"
    }
    {{ #if (or inputs.enable_helm inputs.enable_public_ecr_helm) }}
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
{{ #if inputs.enable_public_ecr_helm }}

provider "aws" {
  region = "us-east-1"
  alias  = "virginia"
}

# Public ECR is only available in us-east-1
data "aws_ecrpublic_authorization_token" "token" {
  provider = aws.virginia
}

provider "helm" {
  alias  = "public_ecr"

  kubernetes {
    host                   = module.eks.cluster_endpoint
    cluster_ca_certificate = base64decode(module.eks.cluster_certificate_authority_data)

    exec {
      api_version = "client.authentication.k8s.io/v1beta1"
      command     = "aws"
      # This requires the awscli to be installed locally where Terraform is executed
      args = ["eks", "get-token", "--cluster-name", module.eks.cluster_name]
    }
    {{ #if inputs.enable_karpenter }}

    registry {
      url      = "oci://public.ecr.aws/karpenter"
      username = data.aws_ecrpublic_authorization_token.token.user_name
      password = data.aws_ecrpublic_authorization_token.token.password
    }
    {{ /if }}
    {{ #if inputs.enable_neuron_devices }}

    registry {
      url      = "oci://public.ecr.aws/neuron"
      username = data.aws_ecrpublic_authorization_token.token.user_name
      password = data.aws_ecrpublic_authorization_token.token.password
    }
    {{ /if }}
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
