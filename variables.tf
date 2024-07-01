variable "on_demand_capacity_reservation_arns" {
  description = "List of the on-demand capacity reservations ARNs to associate with the node group"
  type        = list(string)
  default     = []
}
