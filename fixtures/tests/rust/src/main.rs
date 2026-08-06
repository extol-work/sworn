//! sworn-http-conformance: HTTP conformance test runner.
//!
//! Exercises SPEC §10.2 tests T-2 through T-7 against a live SWORN
//! implementation exposed over HTTP. T-1 and T-5 are covered by the
//! vector-check runners at fixtures/runners/; T-8 (notarizer independent
//! recomputation) is exercised implicitly by T-3; T-9 (no substrate mutation)
//! is inspected out-of-band per implementation.
//!
//! Usage: sworn-http-conformance <api-url>
//!
//! Reference target: sworn-postgres. Any conforming Signer or Notarizer
//! implementation exposing the reference HTTP shape (POST /attestations,
//! GET /attestations) should pass all tests. Implementations with a
//! different wire shape can port the test logic while preserving the
//! assertions.
//!
//! Deliberately no default URL. Silent-run-against-wrong-implementation
//! is worse than a usage error.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use rand_core::{OsRng, RngCore};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::process::ExitCode;

const USAGE: &str = "\
usage: sworn-http-conformance <api-url>

example:
  sworn-http-conformance http://localhost:8080

The URL is required. No default is provided; a runner that
silently exercises the wrong implementation is worse than one
that refuses.
";

// ─── Canonical byte serialization (v0.1-final, 248 bytes) ─────────
//
// Duplicated from fixtures/runners/rust/ intentionally: this runner
// must not depend on the same code the spec text is being tested
// against. Any drift here would surface as a per-test failure.

const CANONICAL_BYTES_LEN: usize = 248;
const SPEC_VERSION_V0_1_FINAL: u16 = 2;

#[derive(Clone, Copy)]
struct Fields {
    signer: [u8; 32],
    subject: [u8; 32],
    activity_hash: [u8; 32],
    data_hash: [u8; 32],
    witness_for: [u8; 32],
    source_hash: [u8; 32],
    source_type: u16,
    confidence: u16,
    witnessing_depth: u8,
    attestor_relationship: u8,
    signer_asserted_at: i64,
    retention_hint: i64,
    nonce: [u8; 32],
}

fn canonical_bytes(f: &Fields) -> [u8; CANONICAL_BYTES_LEN] {
    let mut out = [0u8; CANONICAL_BYTES_LEN];
    let mut o = 0;
    out[o..o + 2].copy_from_slice(&SPEC_VERSION_V0_1_FINAL.to_le_bytes());
    o += 2;
    out[o..o + 32].copy_from_slice(&f.signer);
    o += 32;
    out[o..o + 32].copy_from_slice(&f.subject);
    o += 32;
    out[o..o + 32].copy_from_slice(&f.activity_hash);
    o += 32;
    out[o..o + 32].copy_from_slice(&f.data_hash);
    o += 32;
    out[o..o + 32].copy_from_slice(&f.witness_for);
    o += 32;
    out[o..o + 32].copy_from_slice(&f.source_hash);
    o += 32;
    out[o..o + 2].copy_from_slice(&f.source_type.to_le_bytes());
    o += 2;
    out[o..o + 2].copy_from_slice(&f.confidence.to_le_bytes());
    o += 2;
    out[o] = f.witnessing_depth;
    o += 1;
    out[o] = f.attestor_relationship;
    o += 1;
    out[o..o + 8].copy_from_slice(&f.signer_asserted_at.to_le_bytes());
    o += 8;
    out[o..o + 8].copy_from_slice(&f.retention_hint.to_le_bytes());
    o += 8;
    out[o..o + 32].copy_from_slice(&f.nonce);
    o += 32;
    debug_assert_eq!(o, CANONICAL_BYTES_LEN);
    out
}

// ─── Wire types (mirror sworn-postgres api/src/main.rs) ────────────

#[derive(Debug, Serialize)]
struct CreateReq<'a> {
    signer_pubkey: String,
    subject: String,
    activity_type_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    witness_for: Option<String>,
    source_hash: String,
    source_type: u16,
    confidence: u16,
    witnessing_depth: u8,
    attestor_relationship: u8,
    signer_asserted_at: i64,
    retention_hint: i64,
    nonce: String,
    signature: String,
    payload: &'a serde_json::Value,
}

// No response type: tests only observe HTTP status; response bodies are
// echoed into failure messages verbatim when tests fail.

// ─── Fixture builder ───────────────────────────────────────────────

/// A ready-to-post attestation, plus its wire fields, plus the signing key
/// so tests can mutate one thing at a time before shipping.
struct Fixture {
    fields: Fields,
    signature: [u8; 64],
    activity_type_uri: String,
    payload: serde_json::Value,
    signing_key: SigningKey,
}

impl Fixture {
    /// Build a valid self-reported attestation with defaults suitable for
    /// happy-path testing. Every test starts from a fresh Fixture with a
    /// unique nonce and payload so the server never sees duplicates.
    fn fresh(salt: u64) -> Self {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let signer = signing_key.verifying_key().to_bytes();

        let payload = serde_json::json!({
            "kind": "conformance-test",
            "salt": salt,
        });
        let canonical_payload =
            serde_jcs::to_vec(&payload).expect("payload canonicalizes");
        let data_hash: [u8; 32] = Sha256::digest(&canonical_payload).into();

        let activity_type_uri = "sworn.dev/v1/conformance-test".to_string();
        let activity_hash: [u8; 32] = Sha256::digest(activity_type_uri.as_bytes()).into();

        let mut subject = [0u8; 32];
        OsRng.fill_bytes(&mut subject);
        let mut nonce = [0u8; 32];
        OsRng.fill_bytes(&mut nonce);

        let fields = Fields {
            signer,
            subject,
            activity_hash,
            data_hash,
            witness_for: [0u8; 32],
            source_hash: [0u8; 32],
            source_type: 1, // self_reported
            confidence: 10_000,
            witnessing_depth: 5,     // self_asserted
            attestor_relationship: 1, // self
            signer_asserted_at: 1_780_000_000,
            retention_hint: -1,
            nonce,
        };
        let sig_bytes = signing_key.sign(&canonical_bytes(&fields)).to_bytes();

        Self {
            fields,
            signature: sig_bytes,
            activity_type_uri,
            payload,
            signing_key,
        }
    }

    /// Re-sign after fields have been mutated (used by tests that alter
    /// signed content, so the signature covers the mutation).
    fn resign(&mut self) {
        self.signature = self
            .signing_key
            .sign(&canonical_bytes(&self.fields))
            .to_bytes();
    }

    fn as_request(&self) -> CreateReq<'_> {
        CreateReq {
            signer_pubkey: B64.encode(self.fields.signer),
            subject: B64.encode(self.fields.subject),
            activity_type_uri: self.activity_type_uri.clone(),
            witness_for: None,
            source_hash: B64.encode(self.fields.source_hash),
            source_type: self.fields.source_type,
            confidence: self.fields.confidence,
            witnessing_depth: self.fields.witnessing_depth,
            attestor_relationship: self.fields.attestor_relationship,
            signer_asserted_at: self.fields.signer_asserted_at,
            retention_hint: self.fields.retention_hint,
            nonce: B64.encode(self.fields.nonce),
            signature: B64.encode(self.signature),
            payload: &self.payload,
        }
    }
}

// ─── Test framework ────────────────────────────────────────────────

type TestResult = Result<(), String>;

fn post_attestation(
    http: &reqwest::blocking::Client,
    api_url: &str,
    req: &CreateReq<'_>,
) -> Result<(reqwest::StatusCode, String), String> {
    let url = format!("{}/attestations", api_url.trim_end_matches('/'));
    let resp = http
        .post(&url)
        .json(req)
        .send()
        .map_err(|e| format!("http post failed: {}", e))?;
    let status = resp.status();
    let body = resp
        .text()
        .map_err(|e| format!("read body failed: {}", e))?;
    Ok((status, body))
}

fn expect_status(
    got: reqwest::StatusCode,
    expected: u16,
    body: &str,
) -> TestResult {
    if got.as_u16() == expected {
        Ok(())
    } else {
        Err(format!(
            "expected HTTP {}, got {}: {}",
            expected,
            got.as_u16(),
            body.chars().take(200).collect::<String>()
        ))
    }
}

// ─── Individual tests (SPEC §10.2) ─────────────────────────────────

/// Sanity check: a well-formed attestation is accepted (201). This is not
/// itself a required conformance test but its failure indicates the test
/// fixture is broken and every subsequent test result is meaningless.
fn test_happy_path(http: &reqwest::blocking::Client, api_url: &str) -> TestResult {
    let fx = Fixture::fresh(1);
    let (status, body) = post_attestation(http, api_url, &fx.as_request())?;
    expect_status(status, 201, &body)
}

/// T-4: GET /attestations MUST be refused (400 or similar 4xx).
fn test_t4_refused_list(
    http: &reqwest::blocking::Client,
    api_url: &str,
) -> TestResult {
    let url = format!("{}/attestations", api_url.trim_end_matches('/'));
    let resp = http
        .get(&url)
        .send()
        .map_err(|e| format!("http get failed: {}", e))?;
    let status = resp.status();
    let body = resp
        .text()
        .map_err(|e| format!("read body failed: {}", e))?;
    if !(400..500).contains(&status.as_u16()) {
        return Err(format!(
            "expected 4xx refusal, got {}: {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        ));
    }
    // Reference implementation uses 400 with "refused" in the body; a
    // conforming implementation MAY use any 4xx but should identify the
    // refusal explicitly. Body content is advisory.
    Ok(())
}

/// T-2: signature tamper: flip one byte of the signature, submit, expect 400.
/// The canonical bytes still parse cleanly; only Ed25519 verification fails.
fn test_t2_signature_tamper(
    http: &reqwest::blocking::Client,
    api_url: &str,
) -> TestResult {
    let mut fx = Fixture::fresh(2);
    // Corrupt one byte of the signature: the signature is 64 bytes,
    // flipping the first is sufficient to break Ed25519 verification.
    fx.signature[0] ^= 0xFF;
    let (status, body) = post_attestation(http, api_url, &fx.as_request())?;
    expect_status(status, 400, &body)
}

/// T-3: payload tamper: sign over payload A, submit payload B. The server
/// MUST independently recompute `data_hash` from what it received and
/// reject on mismatch even though the client's signature is arithmetically
/// valid over its own canonical bytes.
///
/// Also exercises T-8 (notarizer independent recomputation) implicitly:
/// an implementation that trusted the client's data_hash would accept
/// this and be non-conforming.
fn test_t3_payload_tamper(
    http: &reqwest::blocking::Client,
    api_url: &str,
) -> TestResult {
    let fx = Fixture::fresh(3);
    // Wire the request with a different payload than the one whose hash
    // is embedded in the signed canonical bytes.
    let mut req = fx.as_request();
    let different_payload = serde_json::json!({
        "kind": "conformance-test",
        "tampered": true,
    });
    req.payload = &different_payload;
    let (status, body) = post_attestation(http, api_url, &req)?;
    expect_status(status, 400, &body)
}

/// T-6a: source_type = self_reported (1) with non-zero source_hash. Per
/// SPEC §2.4, MUST be rejected at verification even if the signature is
/// arithmetically valid over the malformed canonical bytes.
fn test_t6a_sourceless_nonzero_hash_self_reported(
    http: &reqwest::blocking::Client,
    api_url: &str,
) -> TestResult {
    let mut fx = Fixture::fresh(4);
    fx.fields.source_type = 1;
    // Non-zero source_hash: spec-violating for source_type=1.
    fx.fields.source_hash = [0x42u8; 32];
    fx.resign();
    let (status, body) = post_attestation(http, api_url, &fx.as_request())?;
    expect_status(status, 400, &body)
}

/// T-6b: source_type = unknown (0) with non-zero source_hash. Same rule
/// as T-6a applied to the other sourceless enum value.
fn test_t6b_sourceless_nonzero_hash_unknown(
    http: &reqwest::blocking::Client,
    api_url: &str,
) -> TestResult {
    let mut fx = Fixture::fresh(5);
    fx.fields.source_type = 0;
    fx.fields.source_hash = [0x99u8; 32];
    fx.resign();
    let (status, body) = post_attestation(http, api_url, &fx.as_request())?;
    expect_status(status, 400, &body)
}

/// T-7a: source_type value outside the registered range (0..=14). Per
/// SPEC §9.2, implementations MUST fail closed on unknown enum values
/// rather than silently accept them.
fn test_t7a_unknown_source_type(
    http: &reqwest::blocking::Client,
    api_url: &str,
) -> TestResult {
    let mut fx = Fixture::fresh(6);
    fx.fields.source_type = 99;
    // Keep source_hash zero so we're isolating the unknown-enum failure
    // mode rather than combining it with T-6.
    fx.fields.source_hash = [0u8; 32];
    fx.resign();
    let (status, body) = post_attestation(http, api_url, &fx.as_request())?;
    expect_status(status, 400, &body)
}

/// T-7b: witnessing_depth value outside 0..=5. Same fail-closed rule
/// applied to a different provenance enum.
fn test_t7b_unknown_witnessing_depth(
    http: &reqwest::blocking::Client,
    api_url: &str,
) -> TestResult {
    let mut fx = Fixture::fresh(7);
    fx.fields.witnessing_depth = 200;
    fx.resign();
    let (status, body) = post_attestation(http, api_url, &fx.as_request())?;
    expect_status(status, 400, &body)
}

/// T-7c: attestor_relationship value outside 0..=6. Same rule again.
fn test_t7c_unknown_attestor_relationship(
    http: &reqwest::blocking::Client,
    api_url: &str,
) -> TestResult {
    let mut fx = Fixture::fresh(8);
    fx.fields.attestor_relationship = 200;
    fx.resign();
    let (status, body) = post_attestation(http, api_url, &fx.as_request())?;
    expect_status(status, 400, &body)
}

// ─── Runner ────────────────────────────────────────────────────────

struct TestCase {
    label: &'static str,
    run: fn(&reqwest::blocking::Client, &str) -> TestResult,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 || args[1] == "--help" || args[1] == "-h" {
        eprintln!("{}", USAGE);
        return ExitCode::from(2);
    }
    let api_url = args[1].clone();

    let http = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to build http client: {}", e);
            return ExitCode::from(1);
        }
    };

    // Pre-flight: reject unreachable server rather than reporting
    // 9 identical connect-refused errors.
    let health_url = format!("{}/healthz", api_url.trim_end_matches('/'));
    if let Err(e) = http.get(&health_url).send() {
        eprintln!("cannot reach {}: {}", health_url, e);
        eprintln!("is the server up? try: docker compose up -d");
        return ExitCode::from(1);
    }

    println!("sworn-http-conformance: SPEC §10.2 against {}", api_url);
    println!();

    let tests: &[TestCase] = &[
        TestCase { label: "pre-flight: happy-path attestation accepted (201)",
                   run: test_happy_path },
        TestCase { label: "T-4: GET /attestations refused (4xx)",
                   run: test_t4_refused_list },
        TestCase { label: "T-2: signature tamper rejected (400)",
                   run: test_t2_signature_tamper },
        TestCase { label: "T-3: payload tamper rejected (400) [also T-8]",
                   run: test_t3_payload_tamper },
        TestCase { label: "T-6a: source_type=1 + non-zero source_hash rejected",
                   run: test_t6a_sourceless_nonzero_hash_self_reported },
        TestCase { label: "T-6b: source_type=0 + non-zero source_hash rejected",
                   run: test_t6b_sourceless_nonzero_hash_unknown },
        TestCase { label: "T-7a: unknown source_type (99) rejected",
                   run: test_t7a_unknown_source_type },
        TestCase { label: "T-7b: unknown witnessing_depth (200) rejected",
                   run: test_t7b_unknown_witnessing_depth },
        TestCase { label: "T-7c: unknown attestor_relationship (200) rejected",
                   run: test_t7c_unknown_attestor_relationship },
    ];

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<(String, String)> = Vec::new();

    for tc in tests {
        match (tc.run)(&http, &api_url) {
            Ok(()) => {
                println!("  ✓ {}", tc.label);
                passed += 1;
            }
            Err(msg) => {
                println!("  ✗ {}", tc.label);
                failures.push((tc.label.to_string(), msg));
                failed += 1;
            }
        }
    }

    println!();
    println!("{}/{} passed.", passed, passed + failed);

    if failed > 0 {
        println!();
        println!("Failures:");
        for (label, msg) in &failures {
            println!("  {}", label);
            println!("    {}", msg);
        }
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
