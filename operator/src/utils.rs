use argon2::Argon2;
use base64::{engine::general_purpose, Engine};
use bech32::ToBase32;
use kube::{
    api::{Patch, PatchParams},
    core::DynamicObject,
    discovery::ApiResource,
    Api, Client, ResourceExt,
};
use lazy_static::lazy_static;
use serde_json::json;
use std::collections::HashMap;

use crate::{get_config, Error, UtxoRpcPort};

lazy_static! {
    static ref LEGACY_NETWORKS: HashMap<&'static str, String> = {
        let mut m = HashMap::new();
        m.insert("mainnet", "cardano-mainnet".into());
        m.insert("preprod", "cardano-preprod".into());
        m.insert("preview", "cardano-preview".into());
        m
    };
}

pub async fn patch_resource_status(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    name: &str,
    payload: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let status = json!({ "status": payload });
    let patch_params = PatchParams::default();
    api.patch_status(name, &patch_params, &Patch::Merge(status))
        .await?;
    Ok(())
}

pub fn handle_legacy_networks(network: &str) -> String {
    let default = network.to_string();
    LEGACY_NETWORKS.get(network).unwrap_or(&default).to_string()
}

pub fn build_hostname(key: &str, network: &str) -> (String, String) {
    let hostname = get_config()
        .extension_url_per_network
        .get(network)
        .unwrap() // Risky but we know it's there
        .to_string();
    let hostname_key = format!("{key}.{hostname}");

    (hostname, hostname_key)
}

pub async fn build_api_key(crd: &UtxoRpcPort) -> Result<String, Error> {
    let config = get_config();

    let namespace = crd.namespace().unwrap();
    let network = &crd.spec.network;
    let version = crd
        .spec
        .utxorpc_version
        .clone()
        .unwrap_or(config.default_utxorpc_version.clone());

    let name = format!("utxorpc-auth-{}", &crd.name_any());
    let password = format!("{}{}", name, namespace).as_bytes().to_vec();

    let salt = config.api_key_salt.as_bytes();

    let mut output = vec![0; 8];

    let argon2 = Argon2::default();
    let _ = argon2.hash_password_into(password.as_slice(), salt, &mut output);

    let base64 = general_purpose::URL_SAFE_NO_PAD.encode(output);
    let with_bech = bech32::encode(
        &format!("dmtr_utxorpc_{version}_{network}_"),
        base64.to_base32(),
        bech32::Variant::Bech32,
    )
    .unwrap();

    Ok(with_bech)
}

#[cfg(test)]
mod test {
    use std::env;

    use crate::UtxoRpcPortSpec;

    use super::*;

    fn set_configs() {
        env::set_var("DNS_ZONE", "dns_zone");
        env::set_var("EXTENSION_SUBDOMAIN", "extension_subdomain");
        env::set_var("API_KEY_SALT", "api_key_salt");
        env::set_var("METRICS_DELAY", "100");
        env::set_var("PROMETHEUS_URL", "prometheus_url");
        env::set_var("DCU_PER_REQUEST", "preview=5,preprod=5,mainnet=5");
        env::set_var("DEFAULT_UTXORPC_VERSION", "v1");
    }

    #[tokio::test]
    async fn test_build_api_key() {
        set_configs();
        let mut crd = UtxoRpcPort::new(
            "",
            UtxoRpcPortSpec {
                auth_token: None,
                operator_version: Some("1".to_string()),
                network: "preview".to_string(),
                throughput_tier: Some("0".to_string()),
                utxorpc_version: Some("v1".to_string()),
            },
        );
        crd.metadata.namespace = Some("namespace".to_string());

        let api_key = build_api_key(&crd).await.unwrap();
        assert!(api_key.starts_with("dmtr_utxorpc_v1_preview_"));
        assert!(api_key.len() <= 63);
    }
    #[tokio::test]
    async fn test_build_hostname() {
        set_configs();
        let key = "dmtr_utxorpc_v1_preview_ashjdcnoasdj";
        let (hostname, hostname_key) = build_hostname(key, "mainnet");

        assert_eq!(hostname, "mainnet-extension_subdomain.dns_zone".to_string());
        assert_eq!(
            hostname_key,
            "dmtr_utxorpc_v1_preview_ashjdcnoasdj.mainnet-extension_subdomain.dns_zone".to_string()
        );
    }
}
