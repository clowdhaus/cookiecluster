{{ #if (and inputs.enable_compute_reservation (not inputs.enable_karpenter)) }}
################################################################################
# Variables - Required input
################################################################################
{{ #if inputs.enable_odcr }}

variable "on_demand_capacity_reservation_arns" {
  description = "List of the on-demand capacity reservations ARNs to associate with the node group"
  type        = list(string)
}
{{ else if inputs.enable_ml_cbr }}

variable "capacity_reservation_id" {
  description = "The ID of the ML capacity block reservation in which to run the instance(s)"
  type        = string
}
{{ /if }}
{{ /if }}
