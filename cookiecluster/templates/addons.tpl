  cluster_addons = {
  {{ #each add_ons as |a| }}
    {{ a.name }} = {{ #if a.configuration.service_account_role_arn }}{
      service_account_role_arn = {{ a.configuration.service_account_role_arn }}
    }{{ else if (and (eq a.name "coredns") (eq ../inputs.compute_scaling "karpenter")) }}{
      configuration_values = jsonencode({
        tolerations = [
          # Allow CoreDNS to run on the same nodes as the Karpenter controller
          # for use during cluster creation when Karpenter nodes do not yet exist
          {
            key    = "karpenter.sh/controller"
            value  = "true"
            effect = "NoSchedule"
          }
        ]
      })
    }
    {{ ~else }}{}{{ /if }}
  {{ /each}}
  }
