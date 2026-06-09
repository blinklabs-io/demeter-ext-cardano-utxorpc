use std::{collections::HashMap, env};

#[derive(Debug, Clone)]
pub struct Config {
    pub proxy_addr: String,
    pub proxy_namespace: String,
    pub prometheus_addr: String,
    pub ssl_crt_path: String,
    pub ssl_key_path: String,
    pub utxorpc_instances: HashMap<String, String>,
    pub health_endpoint: String,
    pub health_network: String,
    pub health_pool_interval: std::time::Duration,
    pub proxy_tiers_path: std::path::PathBuf,
    pub proxy_tiers_poll_interval: std::time::Duration,
}
impl Config {
    pub fn new() -> Self {
        let utxorpc_instances =
            env::var("UTXORPC_INSTANCES").expect("UTXORPC_INSTANCES must be set");
        let utxorpc_instances = serde_json::from_str::<HashMap<String, String>>(&utxorpc_instances)
            .expect("UTXORPC_INSTANCES must be valid JSON map of network to host:port");

        Self {
            proxy_addr: env::var("PROXY_ADDR").expect("PROXY_ADDR must be set"),
            proxy_namespace: env::var("PROXY_NAMESPACE").expect("PROXY_NAMESPACE must be set"),
            prometheus_addr: env::var("PROMETHEUS_ADDR").expect("PROMETHEUS_ADDR must be set"),
            ssl_crt_path: env::var("SSL_CRT_PATH").expect("SSL_CRT_PATH must be set"),
            ssl_key_path: env::var("SSL_KEY_PATH").expect("SSL_KEY_PATH must be set"),
            utxorpc_instances,
            health_endpoint: "/dmtr_health".to_string(),
            health_network: env::var("HEALTH_NETWORK").unwrap_or("cardano-mainnet".to_string()),
            health_pool_interval: std::time::Duration::from_secs(10),
            proxy_tiers_path: env::var("PROXY_TIERS_PATH")
                .map(|v| v.into())
                .expect("PROXY_TIERS_PATH must be set"),
            proxy_tiers_poll_interval: env::var("PROXY_TIERS_POLL_INTERVAL")
                .map(|v| {
                    std::time::Duration::from_secs(
                        v.parse::<u64>()
                            .expect("PROXY_TIERS_POLL_INTERVAL must be a number in seconds. eg: 2"),
                    )
                })
                .unwrap_or(std::time::Duration::from_secs(2)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
