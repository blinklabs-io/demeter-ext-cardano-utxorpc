use async_trait::async_trait;
use bytes::Bytes;
use pingora::proxy::{ProxyHttp, Session};
use pingora::Result;
use pingora::{http::ResponseHeader, upstreams::peer::HttpPeer};
use pingora_limits::rate::Rate;
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use crate::{Consumer, State, Tier};

/// gRPC status code returned when a consumer exceeds its tier rate limit.
/// See https://grpc.github.io/grpc/core/md_doc_statuscodes.html
static GRPC_STATUS_RESOURCE_EXHAUSTED: &str = "8";

static DMTR_API_KEY: &str = "dmtr-api-key";

pub struct UtxoRpcProxy {
    state: Arc<State>,
    config: Arc<Config>,
}
impl UtxoRpcProxy {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        Self { state, config }
    }

    fn extract_key(&self, session: &Session) -> String {
        session
            .get_header(DMTR_API_KEY)
            .map(|v| v.to_str().unwrap())
            .unwrap_or_default()
            .to_string()
    }

    fn upstream_instance(&self, network: &str) -> Option<&str> {
        self.config
            .utxorpc_instances
            .get(network)
            .map(String::as_str)
    }

    async fn respond_health(&self, session: &mut Session, ctx: &mut Context) {
        ctx.is_health_request = true;
        session.set_keepalive(None);

        let is_healthy = *self.state.upstream_health.read().await;
        let (code, message) = if is_healthy {
            (200, "OK")
        } else {
            (500, "UNHEALTHY")
        };

        let header = Box::new(ResponseHeader::build(code, None).unwrap());
        session.write_response_header(header, true).await.unwrap();
        session
            .write_response_body(Some(Bytes::from(message)), true)
            .await
            .unwrap();
    }

    /// Reject an over-limit gRPC call with a Trailers-Only response carrying
    /// `grpc-status: 8` (RESOURCE_EXHAUSTED). A bare HTTP 429 would be remapped by
    /// gRPC clients to UNAVAILABLE ("retry, server down"), which is the wrong
    /// semantics for a quota breach.
    async fn respond_resource_exhausted(&self, session: &mut Session) {
        let mut header = Box::new(ResponseHeader::build(200, None).unwrap());
        header
            .insert_header("content-type", "application/grpc")
            .unwrap();
        header
            .insert_header("grpc-status", GRPC_STATUS_RESOURCE_EXHAUSTED)
            .unwrap();
        header
            .insert_header("grpc-message", "rate limit exceeded")
            .unwrap();
        // No body: single end-of-stream HEADERS frame (gRPC "Trailers-Only").
        session.write_response_header(header, true).await.unwrap();
    }

    async fn has_limiter(&self, consumer: &Consumer) -> bool {
        let rate_limiter_map = self.state.limiter.read().await;
        rate_limiter_map.get(&consumer.key).is_some()
    }

    async fn add_limiter(&self, consumer: &Consumer, tier: &Tier) {
        let rates = tier
            .rates
            .iter()
            .map(|r| (r.clone(), Rate::new(r.interval)))
            .collect();

        self.state
            .limiter
            .write()
            .await
            .insert(consumer.key.clone(), rates);
    }

    async fn limiter(&self, consumer: &Consumer) -> Result<bool> {
        let tiers = self.state.tiers.read().await.clone();
        let tier = tiers.get(&consumer.tier);
        if tier.is_none() {
            return Ok(true);
        }
        let tier = tier.unwrap();

        if !self.has_limiter(consumer).await {
            self.add_limiter(consumer, tier).await;
        }

        let rate_limiter_map = self.state.limiter.read().await;
        let rates = rate_limiter_map.get(&consumer.key).unwrap();

        if rates
            .iter()
            .any(|(t, r)| r.observe(&consumer.key, 1) > t.limit)
        {
            return Ok(true);
        }

        Ok(false)
    }
}

#[derive(Debug, Default)]
pub struct Context {
    instance: String,
    consumer: Consumer,
    is_health_request: bool,
    rate_limited: bool,
}

#[async_trait]
impl ProxyHttp for UtxoRpcProxy {
    type CTX = Context;
    fn new_ctx(&self) -> Self::CTX {
        Context::default()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        let path = session.req_header().uri.path();
        if path == self.config.health_endpoint {
            self.respond_health(session, ctx).await;
            return Ok(true);
        }

        let key = self.extract_key(session);

        ctx.consumer = match self.state.get_consumer(&key).await {
            Some(consumer) => consumer,
            None => {
                return session.respond_error(401).await.map(|_| true);
            }
        };

        let Some(instance) = self.upstream_instance(&ctx.consumer.network) else {
            return session.respond_error(502).await.map(|_| true);
        };

        ctx.instance = instance.to_string();

        if self.limiter(&ctx.consumer).await? {
            ctx.rate_limited = true;
            self.respond_resource_exhausted(session).await;
            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let mut peer = Box::new(HttpPeer::new(&ctx.instance, false, String::default()));
        peer.options.alpn = pingora::upstreams::peer::ALPN::H2;
        Ok(peer)
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        if !ctx.is_health_request {
            // A rate-limit rejection is a gRPC Trailers-Only response with HTTP status
            // 200 (grpc-status: 8), so the HTTP status alone would hide it. Report it
            // as 429 in metrics/logs to keep enforcement visible in dashboards.
            let response_code = if ctx.rate_limited {
                429
            } else {
                session
                    .response_written()
                    .map_or(0, |resp| resp.status.as_u16())
            };

            info!(
                "{} response code: {response_code}",
                self.request_summary(session, ctx)
            );

            self.state.metrics.inc_http_total_request(
                &ctx.consumer,
                &self.config.proxy_namespace,
                &ctx.instance,
                &response_code,
            );
        }
    }
}
