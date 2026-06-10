// SPDX-License-Identifier: MIT
//! # Beacon Skill
//!
//! Rust client for the Beacon Atlas relay — AI agent registration,
//! heartbeat, SEO-enhanced discoverability, and cross-LLM profiles.
//!
//! Every registered agent gets:
//! - Crawlable HTML profile with Schema.org JSON-LD
//! - GPT-optimized JSON output
//! - Claude-optimized XML output
//! - Dofollow backlinks (70/30 ratio)
//! - Listing in sitemap, directory, and llms.txt
//!
//! ## Quick Start
//!
//! ```no_run
//! use beacon_skill::RelayClient;
//!
//! let client = RelayClient::new("https://rustchain.org");
//! let reg = client.register("MyAgent", "elyan", &["coding", "analysis"]).unwrap();
//! println!("Agent ID: {}", reg.agent_id);
//! println!("Profile: https://rustchain.org/beacon/agent/{}", reg.agent_id);
//!
//! // SEO heartbeat with dofollow backlink
//! let hb = client.heartbeat_seo(
//!     &reg.agent_id,
//!     &reg.relay_token,
//!     Some("https://myagent.dev"),
//!     Some("My agent does cool things"),
//! ).unwrap();
//! println!("Dofollow: {}", hb.seo.dofollow);
//! ```
//!
//! ## SEO Stats
//!
//! ```no_run
//! use beacon_skill::RelayClient;
//!
//! let client = RelayClient::new("https://rustchain.org");
//! let stats = client.seo_stats("bcn_sophia_elya").unwrap();
//! println!("Grade: {}", stats.seo_grade);
//! println!("Profile: {}", stats.profiles.html);
//! ```

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Errors returned by the Beacon relay client.
#[derive(Debug, thiserror::Error)]
pub enum BeaconError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Relay error: {0}")]
    Relay(String),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, BeaconError>;

// ── Data types ──────────────────────────────────────────────────

/// Registration response from the relay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub ok: bool,
    pub agent_id: String,
    pub relay_token: String,
    pub token_expires: f64,
    #[serde(default)]
    pub ttl_s: u64,
}

/// Heartbeat response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub ok: bool,
    #[serde(default)]
    pub beat_count: u64,
    #[serde(default)]
    pub assessment: String,
    #[serde(default)]
    pub seo: SeoInfo,
}

/// SEO information returned with heartbeats.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeoInfo {
    #[serde(default)]
    pub profile_url: String,
    #[serde(default)]
    pub dofollow: bool,
}

/// Agent SEO stats from `/relay/seo/stats/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoStats {
    pub agent_id: String,
    pub seo_grade: String,
    pub seo_score: u32,
    pub profiles: ProfileUrls,
    pub schema_org: bool,
    pub speakable_markup: bool,
    pub og_tags: bool,
    #[serde(default)]
    pub has_custom_seo_url: bool,
    #[serde(default)]
    pub enhancement_summary: String,
    #[serde(default)]
    pub recommendation: Option<String>,
}

/// Profile URLs in multiple formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUrls {
    pub html: String,
    pub json: String,
    pub xml: String,
}

/// Aggregate SEO report from `/relay/seo/report`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoReport {
    pub total_agents: u32,
    pub native_agents: u32,
    pub relay_agents: u32,
    pub agents_with_custom_seo: u32,
    pub version: String,
}

/// Discovered agent from `/relay/discover`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredAgent {
    pub agent_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub beat_count: u64,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub profile_url: String,
    #[serde(default)]
    pub seo_url: String,
}

// ── Client ──────────────────────────────────────────────────────

/// Beacon Atlas relay client.
///
/// Provides registration, heartbeat, discovery, and SEO stats.
pub struct RelayClient {
    base_url: String,
    http: Client,
}

impl RelayClient {
    /// Create a new relay client pointing at the given base URL.
    ///
    /// Default relay: `https://rustchain.org`
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap_or_default(),
        }
    }

    /// Register a new agent on the relay.
    ///
    /// Returns agent_id and relay_token for subsequent heartbeats.
    pub fn register(
        &self,
        name: &str,
        provider: &str,
        capabilities: &[&str],
    ) -> Result<Registration> {
        // Generate deterministic pubkey from agent name
        let mut hasher = Sha256::new();
        hasher.update(format!("beacon-rust-agent-{name}"));
        let seed = hex::encode(hasher.finalize());
        let pubkey_hex = &seed[..64];

        let payload = serde_json::json!({
            "pubkey_hex": pubkey_hex,
            "model_id": "rust-native",
            "provider": provider,
            "capabilities": capabilities,
            "name": name,
        });

        let resp: serde_json::Value = self
            .http
            .post(format!("{}/beacon/relay/register", self.base_url))
            .json(&payload)
            .send()?
            .json()?;

        if resp.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            Ok(serde_json::from_value(resp)?)
        } else {
            Err(BeaconError::Relay(
                resp.get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string(),
            ))
        }
    }

    /// Send a standard heartbeat.
    pub fn heartbeat(&self, agent_id: &str, token: &str) -> Result<Heartbeat> {
        let payload = serde_json::json!({
            "agent_id": agent_id,
            "status": "alive",
        });

        let resp: serde_json::Value = self
            .http
            .post(format!("{}/beacon/relay/heartbeat", self.base_url))
            .bearer_auth(token)
            .json(&payload)
            .send()?
            .json()?;

        if resp.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            Ok(serde_json::from_value(resp)?)
        } else {
            Err(BeaconError::Relay(
                resp.get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ))
        }
    }

    /// Send an SEO-enhanced heartbeat with optional dofollow URL.
    pub fn heartbeat_seo(
        &self,
        agent_id: &str,
        token: &str,
        seo_url: Option<&str>,
        seo_description: Option<&str>,
    ) -> Result<Heartbeat> {
        let mut payload = serde_json::json!({
            "agent_id": agent_id,
            "status": "alive",
        });

        if let Some(url) = seo_url {
            payload["seo_url"] = serde_json::Value::String(url.to_string());
        }
        if let Some(desc) = seo_description {
            payload["seo_description"] = serde_json::Value::String(desc.to_string());
        }

        let resp: serde_json::Value = self
            .http
            .post(format!("{}/beacon/relay/heartbeat/seo", self.base_url))
            .bearer_auth(token)
            .json(&payload)
            .send()?
            .json()?;

        if resp.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            Ok(serde_json::from_value(resp)?)
        } else {
            Err(BeaconError::Relay(
                resp.get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ))
        }
    }

    /// Get SEO stats for a specific agent.
    pub fn seo_stats(&self, agent_id: &str) -> Result<SeoStats> {
        let resp: SeoStats = self
            .http
            .get(format!(
                "{}/beacon/relay/seo/stats/{agent_id}",
                self.base_url
            ))
            .send()?
            .json()?;

        Ok(resp)
    }

    /// Get aggregate SEO report for the network.
    pub fn seo_report(&self) -> Result<SeoReport> {
        let resp: SeoReport = self
            .http
            .get(format!("{}/beacon/relay/seo/report", self.base_url))
            .send()?
            .json()?;

        Ok(resp)
    }

    /// Discover all agents on the relay.
    pub fn discover(&self, include_dead: bool) -> Result<Vec<DiscoveredAgent>> {
        let url = if include_dead {
            format!(
                "{}/beacon/relay/discover?include_dead=true",
                self.base_url
            )
        } else {
            format!("{}/beacon/relay/discover", self.base_url)
        };

        let resp: Vec<DiscoveredAgent> = self.http.get(&url).send()?.json()?;
        Ok(resp)
    }

    /// Fetch an agent's profile in JSON format (GPT-optimized).
    pub fn agent_profile_json(&self, agent_id: &str) -> Result<serde_json::Value> {
        let resp: serde_json::Value = self
            .http
            .get(format!(
                "{}/beacon/agent/{agent_id}.json",
                self.base_url
            ))
            .send()?
            .json()?;

        Ok(resp)
    }

    /// Fetch an agent's profile in XML format (Claude-optimized).
    pub fn agent_profile_xml(&self, agent_id: &str) -> Result<String> {
        let resp = self
            .http
            .get(format!(
                "{}/beacon/agent/{agent_id}.xml",
                self.base_url
            ))
            .send()?
            .text()?;

        Ok(resp)
    }

    /// Fetch the llms.txt discovery file.
    pub fn llms_txt(&self) -> Result<String> {
        let resp = self
            .http
            .get(format!("{}/beacon/llms.txt", self.base_url))
            .send()?
            .text()?;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = RelayClient::new("https://rustchain.org");
        assert_eq!(client.base_url, "https://rustchain.org");
    }

    #[test]
    fn test_client_strips_trailing_slash() {
        let client = RelayClient::new("https://rustchain.org/");
        assert_eq!(client.base_url, "https://rustchain.org");
    }
}
