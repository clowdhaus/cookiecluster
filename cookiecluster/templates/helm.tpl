{{ #if (or inputs.enable_helm inputs.enable_public_ecr_helm) }}
{{ #if inputs.enable_karpenter }}
################################################################################
# Karpenter Controller & Node IAM roles, SQS Queue, and Eventbridge Rules
################################################################################

module "karpenter" {
  source  = "terraform-aws-modules/eks/aws//modules/karpenter"
  version = "~> 21.10"

  cluster_name = module.eks.cluster_name

  # Name needs to match role name passed to the EC2NodeClass
  node_iam_role_use_name_prefix   = false
  node_iam_role_name              = "{{ inputs.name }}-karpenter-node"

  # Avoid policy size limit error
  enable_inline_policy = true

  tags = module.tags.tags
}

################################################################################
# Karpenter Helm chart
################################################################################

resource "helm_release" "karpenter" {
  name       = "karpenter"
  namespace  = "kube-system"
  repository = "oci://public.ecr.aws/karpenter"
  chart      = "karpenter"
  version    = "1.8.3"
  wait       = false

  values = [
    <<-EOT
      dnsPolicy: Default
      serviceAccount:
        name: ${module.karpenter.service_account}
      settings:
        clusterName: ${module.eks.cluster_name}
        clusterEndpoint: ${module.eks.cluster_endpoint}
        interruptionQueue: ${module.karpenter.queue_name}
    EOT
  ]
}

{{ /if }}
################################################################################
# Helm charts
################################################################################
{{ #if inputs.enable_nvidia_gpus }}

resource "helm_release" "nvidia_device_plugin" {
  name             = "nvidia-device-plugin"
  repository       = "https://nvidia.github.io/k8s-device-plugin"
  chart            = "nvidia-device-plugin"
  version          = "0.18.0"
  namespace        = "nvidia-device-plugin"
  create_namespace = true
  wait             = false
}
{{ /if }}
{{ #if inputs.enable_neuron_devices }}

resource "helm_release" "neuron" {
  name             = "neuron"
  repository       = "oci://public.ecr.aws/neuron"
  chart            = "neuron-helm-chart"
  version          = "1.4.0"
  namespace        = "neuron"
  create_namespace = true
  wait             = false

  values = [
    <<-EOT
      nodeSelector:
        aws.amazon.com/neuron.present: 'true'
      npd:
        enabled: false
    EOT
  ]
}
{{ /if }}
{{ #if inputs.enable_efa }}

resource "helm_release" "aws_efa_device_plugin" {
  name       = "aws-efa-k8s-device-plugin"
  namespace  = "kube-system"
  repository = "https://aws.github.io/eks-charts"
  chart      = "aws-efa-k8s-device-plugin"
  version    = "v0.5.20"
  wait       = false

  values = [
    <<-EOT
      nodeSelector:
        vpc.amazonaws.com/efa.present: 'true'
      {{ #if inputs.enable_nvidia_gpus }}
      tolerations:
        - key: nvidia.com/gpu
          operator: Exists
          effect: NoSchedule
      {{ /if }}
      {{ #if inputs.enable_neuron_devices }}
      tolerations:
        - key: aws.amazon.com/neuron
          operator: Exists
          effect: NoSchedule
      {{ /if }}
    EOT
  ]
}
{{ /if }}
{{ /if }}
