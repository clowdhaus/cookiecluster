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
