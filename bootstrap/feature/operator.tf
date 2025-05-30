locals {
  port = 9946
}

resource "kubernetes_deployment_v1" "operator" {
  wait_for_rollout = false

  metadata {
    namespace = var.namespace
    name      = "operator"
    labels = {
      role = "operator"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        role = "operator"
      }
    }

    template {
      metadata {
        labels = {
          role = "operator"
        }
      }

      spec {
        container {
          image = "ghcr.io/demeter-run/ext-cardano-utxorpc-operator:${var.operator_image_tag}"
          name  = "main"

          env {
            name  = "ADDR"
            value = "0.0.0.0:${local.port}"
          }

          env {
            name  = "API_KEY_SALT"
            value = var.api_key_salt
          }

          env {
            name  = "EXTENSION_URL_PER_NETWORK"
            value = join(",", [for k, v in var.extension_urls_per_network : "${k}=${v[0]}"])
          }

          env {
            name  = "PROMETHEUS_URL"
            value = var.prometheus_url
          }

          env {
            name  = "METRICS_DELAY"
            value = var.metrics_delay
          }

          env {
            name  = "METRICS_STEP"
            value = var.metrics_step
          }

          resources {
            limits = {
              cpu    = "4"
              memory = "256Mi"
            }
            requests = {
              cpu    = "50m"
              memory = "256Mi"
            }
          }

          port {
            name           = "metrics"
            container_port = local.port
            protocol       = "TCP"
          }
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-profile"
          operator = "Exists"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-arch"
          operator = "Equal"
          value    = "x86"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/availability-sla"
          operator = "Equal"
          value    = "consistent"
        }
      }
    }
  }
}

