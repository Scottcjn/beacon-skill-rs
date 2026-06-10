// SPDX-License-Identifier: MIT
use beacon_skill::{DiscoveredAgent, Heartbeat, SeoReport, SeoStats};

#[test]
fn heartbeat_deserializes_missing_optional_fields_to_safe_defaults() {
    let heartbeat: Heartbeat = serde_json::from_value(serde_json::json!({
        "ok": true
    }))
    .expect("minimal heartbeat should deserialize");

    assert!(heartbeat.ok);
    assert_eq!(heartbeat.beat_count, 0);
    assert_eq!(heartbeat.assessment, "");
    assert_eq!(heartbeat.seo.profile_url, "");
    assert!(!heartbeat.seo.dofollow);
}

#[test]
fn discovered_agent_deserializes_missing_metadata_to_empty_values() {
    let agent: DiscoveredAgent = serde_json::from_value(serde_json::json!({
        "agent_id": "bcn_test"
    }))
    .expect("minimal discovered agent should deserialize");

    assert_eq!(agent.agent_id, "bcn_test");
    assert_eq!(agent.name, "");
    assert_eq!(agent.provider, "");
    assert_eq!(agent.status, "");
    assert_eq!(agent.beat_count, 0);
    assert!(agent.capabilities.is_empty());
    assert_eq!(agent.profile_url, "");
    assert_eq!(agent.seo_url, "");
}

#[test]
fn seo_stats_defaults_newer_fields_when_relay_omits_them() {
    let stats: SeoStats = serde_json::from_value(serde_json::json!({
        "agent_id": "bcn_stats",
        "seo_grade": "A",
        "seo_score": 91,
        "profiles": {
            "html": "https://example.test/agent",
            "json": "https://example.test/agent.json",
            "xml": "https://example.test/agent.xml"
        },
        "schema_org": true,
        "speakable_markup": false,
        "og_tags": true
    }))
    .expect("older SEO stats payload should deserialize");

    assert_eq!(stats.agent_id, "bcn_stats");
    assert!(!stats.has_custom_seo_url);
    assert_eq!(stats.enhancement_summary, "");
    assert_eq!(stats.recommendation, None);
}

#[test]
fn seo_report_round_trips_through_json() {
    let report = SeoReport {
        total_agents: 12,
        native_agents: 5,
        relay_agents: 7,
        agents_with_custom_seo: 3,
        version: "2026.05".to_string(),
    };

    let encoded = serde_json::to_string(&report).expect("report should serialize");
    let decoded: SeoReport = serde_json::from_str(&encoded).expect("report should deserialize");

    assert_eq!(decoded.total_agents, 12);
    assert_eq!(decoded.native_agents, 5);
    assert_eq!(decoded.relay_agents, 7);
    assert_eq!(decoded.agents_with_custom_seo, 3);
    assert_eq!(decoded.version, "2026.05");
}
