{{ #if (or (eq inputs.reservation "ODCR") (eq inputs.reservation "CBR")) }}

################################################################################
# Variables - Required input
################################################################################
{{ #if (eq inputs.reservation "ODCR") }}

variable "on_demand_capacity_reservation_arns" {
  description = "List of the on-demand capacity reservations ARNs to associate with the node group"
  type        = list(string)
}
{{ /if }}
{{ #if (eq inputs.reservation "CBR") }}

variable "capacity_reservation_id" {
  description = "The ID of the ML capacity block reservation in which to run the instance(s)"
  type        = string
}
{{ /if }}
{{ /if }}
