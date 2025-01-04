variable "namespace" {
  type = string
}

variable "networks" {
  type    = list(string)
  default = ["mainnet", "preprod", "preview", "vector-testnet"]
}

variable "network_addresses" {
  type    = map(string)
  default = {}
}

// Feature
variable "operator_image_tag" {
  type = string
}

variable "api_key_salt" {
  type = string
}

// Proxies
variable "extension_subdomain" {
  type = string
}

variable "dns_zone" {
  default = "demeter.run"
}

variable "cloud_provider" {
  type    = string
  default = "aws"
}

variable "cluster_issuer" {
  type    = string
  default = "letsencrypt-dns01"
}

variable "proxies_image_tag" {
  type = string
}

variable "proxies_replicas" {
  type    = number
  default = 1
}

# TODO - not in use yet
# variable "proxy_green_image_tag" {
#   type = string
# }

# variable "proxy_green_replicas" {
#   type    = number
#   default = 1
# }

# variable "proxy_blue_image_tag" {
#   type = string
# }

# variable "proxy_blue_replicas" {
#   type    = number
#   default = 1
# }

variable "proxies_resources" {
  type = object({
    limits = object({
      cpu    = string
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    limits : {
      cpu : "2",
      memory : "250Mi"
    }
    requests : {
      cpu : "50m",
      memory : "250Mi"
    }
  }
}

variable "proxies_tolerations" {
  type = list(object({
    effect   = string
    key      = string
    operator = string
    value    = optional(string)
  }))
  default = [
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-profile"
      operator = "Exists"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-arch"
      operator = "Equal"
      value    = "arm64"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/availability-sla"
      operator = "Exists"
    }
  ]
}

// Cloudflared
variable "cloudflared_tunnel_id" {
  type = string
}

variable "cloudflared_tunnel_secret" {
  type        = string
  description = "TunnelSecret, written on credentials file."
}

variable "cloudflared_account_tag" {
  type        = string
  description = "AccountTag, written on credentials file."
}

variable "cloudflared_metrics_port" {
  type    = number
  default = 2000
}

variable "cloudflared_image_tag" {
  type    = string
  default = "latest"
}

variable "cloudflared_replicas" {
  type    = number
  default = 2
}

variable "cloudflared_resources" {
  type = object({
    limits = object({
      cpu    = string
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    limits : {
      cpu : "1",
      memory : "500Mi"
    }
    requests : {
      cpu : "50m",
      memory : "500Mi"
    }
  }
}

variable "cloudflared_tolerations" {
  type = list(object({
    effect   = string
    key      = string
    operator = string
    value    = optional(string)
  }))
  default = [
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-profile"
      operator = "Exists"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-arch"
      operator = "Exists"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/availability-sla"
      operator = "Exists"
    }
  ]
}

variable "cells" {
  type = map(object({
    tolerations = optional(list(object({
      effect   = string
      key      = string
      operator = string
      value    = string
    })))
    pvc = object({
      storage_class = string
      storage_size  = string
      volume_name   = optional(string)
    })
    instances = map(object({
      dolos_version = string
      replicas      = optional(number)
      resources = optional(object({
        limits = object({
          cpu    = string
          memory = string
        })
        requests = object({
          cpu    = string
          memory = string
        })
      }))
    }))
  }))
}
