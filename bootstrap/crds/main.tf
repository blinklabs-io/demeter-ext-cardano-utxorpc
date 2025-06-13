resource "kubernetes_manifest" "customresourcedefinition_utxorpcports_demeter_run" {
  manifest = {
    "apiVersion" = "apiextensions.k8s.io/v1"
    "kind" = "CustomResourceDefinition"
    "metadata" = {
      "name" = "utxorpcports.demeter.run"
    }
    "spec" = {
      "group" = "demeter.run"
      "names" = {
        "categories" = [
          "demeter-port",
        ]
        "kind" = "UtxoRpcPort"
        "plural" = "utxorpcports"
        "shortNames" = [
          "utxoport",
        ]
        "singular" = "utxorpcport"
      }
      "scope" = "Namespaced"
      "versions" = [
        {
          "additionalPrinterColumns" = [
            {
              "jsonPath" = ".spec.network"
              "name" = "Network"
              "type" = "string"
            },
            {
              "jsonPath" = ".spec.throughputTier"
              "name" = "Throughput Tier"
              "type" = "string"
            },
            {
              "jsonPath" = ".spec.utxorpcVersion"
              "name" = "UtxoRPC Version"
              "type" = "string"
            },
            {
              "jsonPath" = ".status.grpcEndpointUrl"
              "name" = "Endpoint"
              "type" = "string"
            },
            {
              "jsonPath" = ".status.authToken"
              "name" = "Auth Token"
              "type" = "string"
            },
          ]
          "name" = "v1alpha1"
          "schema" = {
            "openAPIV3Schema" = {
              "description" = "Auto-generated derived type for UtxoRpcPortSpec via `CustomResource`"
              "properties" = {
                "spec" = {
                  "properties" = {
                    "authToken" = {
                      "nullable" = true
                      "type" = "string"
                    }
                    "network" = {
                      "type" = "string"
                    }
                    "operatorVersion" = {
                      "nullable" = true
                      "type" = "string"
                    }
                    "throughputTier" = {
                      "nullable" = true
                      "type" = "string"
                    }
                    "utxorpcVersion" = {
                      "nullable" = true
                      "type" = "string"
                    }
                  }
                  "required" = [
                    "network",
                  ]
                  "type" = "object"
                }
                "status" = {
                  "nullable" = true
                  "properties" = {
                    "authToken" = {
                      "type" = "string"
                    }
                    "grpcEndpointUrl" = {
                      "type" = "string"
                    }
                  }
                  "required" = [
                    "authToken",
                    "grpcEndpointUrl",
                  ]
                  "type" = "object"
                }
              }
              "required" = [
                "spec",
              ]
              "title" = "UtxoRpcPort"
              "type" = "object"
            }
          }
          "served" = true
          "storage" = true
          "subresources" = {
            "status" = {}
          }
        },
      ]
    }
  }
}
