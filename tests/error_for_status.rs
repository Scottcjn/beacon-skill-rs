// SPDX-License-Identifier: MIT
//! Regression coverage for non-success responses from read endpoints.
//!
//! Without `error_for_status()`, JSON endpoints can return decoded error
//! payloads or decode failures, while text endpoints return successful strings.

use beacon_skill::{BeaconError, RelayClient};
use reqwest::StatusCode;
use std::fmt::Debug;

fn assert_http_status<T: Debug>(result: beacon_skill::Result<T>, expected: StatusCode) {
    match result {
        Err(BeaconError::Http(error)) => {
            assert_eq!(
                error.status(),
                Some(expected),
                "unexpected error: {error:?}"
            );
        }
        other => panic!("expected HTTP {expected} error, got: {other:?}"),
    }
}

#[test]
fn seo_stats_returns_http_error_on_404() {
    let mut server = mockito::Server::new();
    let m = server
        .mock("GET", "/beacon/relay/seo/stats/bcn_missing")
        .with_status(404)
        .with_body("{\"error\":\"not found\"}")
        .create();

    let client = RelayClient::new(&server.url());
    let result = client.seo_stats("bcn_missing");
    m.assert();
    assert_http_status(result, StatusCode::NOT_FOUND);
}

#[test]
fn seo_report_returns_http_error_on_503() {
    let mut server = mockito::Server::new();
    let m = server
        .mock("GET", "/beacon/relay/seo/report")
        .with_status(503)
        .with_body("maintenance")
        .create();

    let client = RelayClient::new(&server.url());
    let result = client.seo_report();
    m.assert();
    assert_http_status(result, StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn discover_returns_http_error_on_500() {
    let mut server = mockito::Server::new();
    let m = server
        .mock("GET", "/beacon/relay/discover")
        .with_status(500)
        .with_body("upstream error")
        .create();

    let client = RelayClient::new(&server.url());
    let result = client.discover(false);
    m.assert();
    assert_http_status(result, StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn agent_profile_json_returns_http_error_on_404() {
    let mut server = mockito::Server::new();
    let m = server
        .mock("GET", "/beacon/agent/missing.json")
        .with_status(404)
        .with_body("{\"error\":\"no such agent\"}")
        .create();

    let client = RelayClient::new(&server.url());
    let result = client.agent_profile_json("missing");
    m.assert();
    assert_http_status(result, StatusCode::NOT_FOUND);
}

#[test]
fn agent_profile_xml_returns_http_error_on_401() {
    let mut server = mockito::Server::new();
    let m = server
        .mock("GET", "/beacon/agent/private.xml")
        .with_status(401)
        .with_body("unauthorized")
        .create();

    let client = RelayClient::new(&server.url());
    let result = client.agent_profile_xml("private");
    m.assert();
    assert_http_status(result, StatusCode::UNAUTHORIZED);
}

#[test]
fn llms_txt_returns_http_error_on_404() {
    let mut server = mockito::Server::new();
    let m = server
        .mock("GET", "/beacon/llms.txt")
        .with_status(404)
        .with_body("missing")
        .create();

    let client = RelayClient::new(&server.url());
    let result = client.llms_txt();
    m.assert();
    assert_http_status(result, StatusCode::NOT_FOUND);
}
