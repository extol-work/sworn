//! SWORN v0.1-final golden-vector conformance runner (Rust).
//!
//! Reads a vectors.json file (see fixtures/attestations/v0.1-final/) and
//! verifies each vector against SPEC §3.1 and §3.2 by:
//!
//!   1. Reconstructing the 248-byte canonical byte sequence from `input_fields`
//!      and comparing to `expected_canonical_bytes_hex`.
//!   2. Signing the canonical bytes with the Ed25519 key derived from
//!      `signer_secret_seed_hex` and comparing to `expected_signature_hex`.
//!   3. Verifying the signature against the reconstructed canonical bytes.
//!
//! Any conforming SWORN implementation MUST reproduce (1) and (2) byte-for-byte.
//! Test (3) is a sanity check against transcription bugs in (1) or (2).
//!
//! Deliberately self-contained: no dependency on sworn-verify or any other
//! implementation crate. If a reader wants to understand SWORN's serialization
//! from a Rust program that isn't the reference implementation, this is it.
//!
//! Usage:
//!
//!     sworn-vector-check <path-to-vectors.json>
//!
//! Exits 0 if every vector passes all three checks. Exits 1 with per-vector
//! diagnostics otherwise. The path argument is required; there is no default
//! (fail-loud discipline; a runner that silently checks the wrong vectors is
//! worse than one that refuses to run).

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Deserialize;
use std::env;
use std::fs;
use std::process::ExitCode;

const CANONICAL_BYTES_LEN: usize = 248;

// ─── Vector schema ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VectorFile {
    spec_version: u16,
    canonical_bytes_length: usize,
    vectors: Vec<Vector>,
}

#[derive(Deserialize)]
struct Vector {
    name: String,
    #[allow(dead_code)]
    notes: String,
    spec_version: u16,
    input_fields: InputFields,
    signer_secret_seed_hex: String,
    expected_canonical_bytes_hex: String,
    expected_canonical_bytes_len: usize,
    expected_signature_hex: String,
}

#[derive(Deserialize)]
struct InputFields {
    signer_hex: String,
    subject_hex: String,
    activity_hash_hex: String,
    data_hash_hex: String,
    witness_for_hex: String,
    source_hash_hex: String,
    source_type: u16,
    confidence: u16,
    witnessing_depth: u8,
    attestor_relationship: u8,
    signer_asserted_at: i64,
    retention_hint: i64,
    nonce_hex: String,
}

// ─── Canonical byte construction (SPEC §3.1) ─────────────────────────
//
// Field-by-field layout of the 248-byte v0.1-final canonical bytes:
//
//     byte   0..  2   spec_version              (u16 little-endian)
//     byte   2.. 34   signer                    (32 bytes)
//     byte  34.. 66   subject                   (32 bytes)
//     byte  66.. 98   activity_hash             (32 bytes)
//     byte  98..130   data_hash                 (32 bytes)
//     byte 130..162   witness_for               (32 bytes)
//     byte 162..194   source_hash               (32 bytes)
//     byte 194..196   source_type               (u16 little-endian)
//     byte 196..198   confidence                (u16 little-endian)
//     byte 198..199   witnessing_depth          (u8)
//     byte 199..200   attestor_relationship     (u8)
//     byte 200..208   signer_asserted_at        (i64 little-endian)
//     byte 208..216   retention_hint            (i64 little-endian)
//     byte 216..248   nonce                     (32 bytes)
//
// Deviation from this layout (endianness, field order, added framing) is a
// spec violation and will cause cross-implementation verification to fail.

fn build_canonical_bytes(spec_version: u16, f: &InputFields) -> Result<[u8; CANONICAL_BYTES_LEN], String> {
    let signer = decode_32(&f.signer_hex, "signer")?;
    let subject = decode_32(&f.subject_hex, "subject")?;
    let activity_hash = decode_32(&f.activity_hash_hex, "activity_hash")?;
    let data_hash = decode_32(&f.data_hash_hex, "data_hash")?;
    let witness_for = decode_32(&f.witness_for_hex, "witness_for")?;
    let source_hash = decode_32(&f.source_hash_hex, "source_hash")?;
    let nonce = decode_32(&f.nonce_hex, "nonce")?;

    // Enforce the SPEC §2.4 sourceless-zero-hash rule: source_type in {0, 1}
    // requires source_hash == 32 zero bytes. A vector that violates this is
    // malformed at Layer 1 and would fail verification independently of
    // signature validity, so we surface it here as a runner-level check.
    if (f.source_type == 0 || f.source_type == 1) && source_hash != [0u8; 32] {
        return Err(format!(
            "source_type = {} requires source_hash = 32 zero bytes (SPEC §2.4)",
            f.source_type
        ));
    }

    let mut out = [0u8; CANONICAL_BYTES_LEN];
    let mut off = 0;

    out[off..off + 2].copy_from_slice(&spec_version.to_le_bytes());
    off += 2;

    for src in [&signer, &subject, &activity_hash, &data_hash, &witness_for, &source_hash] {
        out[off..off + 32].copy_from_slice(src);
        off += 32;
    }

    out[off..off + 2].copy_from_slice(&f.source_type.to_le_bytes());
    off += 2;
    out[off..off + 2].copy_from_slice(&f.confidence.to_le_bytes());
    off += 2;
    out[off] = f.witnessing_depth;
    off += 1;
    out[off] = f.attestor_relationship;
    off += 1;
    out[off..off + 8].copy_from_slice(&f.signer_asserted_at.to_le_bytes());
    off += 8;
    out[off..off + 8].copy_from_slice(&f.retention_hint.to_le_bytes());
    off += 8;
    out[off..off + 32].copy_from_slice(&nonce);
    off += 32;

    debug_assert_eq!(off, CANONICAL_BYTES_LEN);
    Ok(out)
}

// ─── Hex helpers ─────────────────────────────────────────────────────

fn decode_32(s: &str, field: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(s).map_err(|e| format!("{field}: hex decode: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!("{field}: expected 32 bytes, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_64(s: &str, field: &str) -> Result<[u8; 64], String> {
    let bytes = hex::decode(s).map_err(|e| format!("{field}: hex decode: {e}"))?;
    if bytes.len() != 64 {
        return Err(format!("{field}: expected 64 bytes, got {}", bytes.len()));
    }
    let mut out = [0u8; 64];
    out.copy_from_slice(&bytes);
    Ok(out)
}

// ─── Ed25519 signing per SPEC §3.2 ────────────────────────────────────
//
// PureEdDSA per RFC 8032 §5.1. Implementations MUST NOT use Ed25519ph. The
// ed25519-dalek 2.x Signer::sign path is PureEdDSA; do not switch to
// Signer::sign_prehashed or an equivalent, which would silently break
// cross-implementation verification.

fn sign_canonical(seed: &[u8; 32], canonical_bytes: &[u8; CANONICAL_BYTES_LEN]) -> [u8; 64] {
    let sk = SigningKey::from_bytes(seed);
    let sig: Signature = sk.sign(canonical_bytes);
    sig.to_bytes()
}

fn verify_canonical(
    signer_pubkey: &[u8; 32],
    canonical_bytes: &[u8; CANONICAL_BYTES_LEN],
    signature: &[u8; 64],
) -> Result<(), String> {
    let vk = VerifyingKey::from_bytes(signer_pubkey)
        .map_err(|e| format!("verifying key: {e}"))?;
    let sig = Signature::from_bytes(signature);
    vk.verify(canonical_bytes, &sig)
        .map_err(|e| format!("signature: {e}"))
}

// ─── Per-vector diagnostics ──────────────────────────────────────────
//
// Each of the three checks is reported separately so a failure is legible
// per-mode. A canonical-bytes mismatch is a serialization bug (probably in
// field order, endianness, or a missing/extra field). A signature mismatch
// with correct canonical bytes is a signing-path bug (PureEdDSA vs Ed25519ph,
// wrong seed handling, expanded-vs-seed private key confusion). A verify
// failure with correct canonical bytes and correct signature is either a
// verifier-side pubkey handling bug or (in this runner) evidence that
// something else is very wrong.

struct VectorReport {
    name: String,
    canonical_ok: bool,
    signature_ok: bool,
    verify_ok: bool,
    details: Vec<String>,
}

impl VectorReport {
    fn passed(&self) -> bool {
        self.canonical_ok && self.signature_ok && self.verify_ok
    }

    fn status_glyph(ok: bool) -> &'static str {
        if ok { "✓" } else { "✗" }
    }
}

fn run_vector(v: &Vector) -> VectorReport {
    let mut details = Vec::new();

    // (0) Basic sanity: the vector's declared expected length matches the spec.
    if v.expected_canonical_bytes_len != CANONICAL_BYTES_LEN {
        details.push(format!(
            "vector declares expected_canonical_bytes_len = {}, spec §3.1 requires {}",
            v.expected_canonical_bytes_len, CANONICAL_BYTES_LEN
        ));
    }

    // (1) Reconstruct canonical bytes and compare hex-to-hex.
    let (canonical_bytes, canonical_ok) = match build_canonical_bytes(v.spec_version, &v.input_fields) {
        Ok(bytes) => {
            let actual_hex = hex::encode(bytes);
            let expected_hex = v.expected_canonical_bytes_hex.to_lowercase();
            let ok = actual_hex == expected_hex;
            if !ok {
                details.push(format!(
                    "canonical bytes mismatch:\n    expected: {}\n    actual:   {}",
                    expected_hex, actual_hex
                ));
            }
            (Some(bytes), ok)
        }
        Err(e) => {
            details.push(format!("canonical byte construction failed: {}", e));
            (None, false)
        }
    };

    // (2) Sign with the vector's seed and compare to expected signature.
    // (3) Verify the produced signature against the reconstructed bytes.
    let (signature_ok, verify_ok) = if let Some(canonical_bytes) = canonical_bytes {
        let seed = match decode_32(&v.signer_secret_seed_hex, "signer_secret_seed") {
            Ok(s) => s,
            Err(e) => {
                details.push(format!("signer_secret_seed: {}", e));
                return VectorReport {
                    name: v.name.clone(),
                    canonical_ok,
                    signature_ok: false,
                    verify_ok: false,
                    details,
                };
            }
        };

        let produced_sig = sign_canonical(&seed, &canonical_bytes);
        let produced_hex = hex::encode(produced_sig);
        let expected_hex = v.expected_signature_hex.to_lowercase();
        let sig_ok = produced_hex == expected_hex;
        if !sig_ok {
            details.push(format!(
                "signature mismatch:\n    expected: {}\n    actual:   {}",
                expected_hex, produced_hex
            ));
        }

        // Verify against the pubkey embedded in the input_fields.signer.
        let signer_pubkey = decode_32(&v.input_fields.signer_hex, "signer").unwrap();
        let verify_sig = match decode_64(&v.expected_signature_hex, "expected_signature") {
            Ok(s) => s,
            Err(e) => {
                details.push(format!("expected_signature decode: {}", e));
                return VectorReport {
                    name: v.name.clone(),
                    canonical_ok,
                    signature_ok: sig_ok,
                    verify_ok: false,
                    details,
                };
            }
        };
        let verify_ok = match verify_canonical(&signer_pubkey, &canonical_bytes, &verify_sig) {
            Ok(()) => true,
            Err(e) => {
                details.push(format!("verify failed: {}", e));
                false
            }
        };

        (sig_ok, verify_ok)
    } else {
        (false, false)
    };

    VectorReport {
        name: v.name.clone(),
        canonical_ok,
        signature_ok,
        verify_ok,
        details,
    }
}

// ─── main ─────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: sworn-vector-check <path-to-vectors.json>");
        eprintln!();
        eprintln!("example:");
        eprintln!("  sworn-vector-check fixtures/attestations/v0.1-final/vectors.json");
        eprintln!();
        eprintln!("The path is required. No default is provided; a runner that");
        eprintln!("silently checks the wrong vectors is worse than one that refuses.");
        return ExitCode::from(2);
    }

    let path = &args[1];
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot read {}: {}", path, e);
            return ExitCode::from(2);
        }
    };

    let vf: VectorFile = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot parse {} as SWORN vector file: {}", path, e);
            return ExitCode::from(2);
        }
    };

    // Spec-version and length are metadata that let us fail-loud if this
    // runner is pointed at a vector file for a spec revision it doesn't
    // understand.
    if vf.spec_version != 2 {
        eprintln!(
            "this runner is for SPEC v0.1-final (spec_version=2); vectors file declares spec_version={}",
            vf.spec_version
        );
        return ExitCode::from(2);
    }
    if vf.canonical_bytes_length != CANONICAL_BYTES_LEN {
        eprintln!(
            "vectors file declares canonical_bytes_length={}; this runner enforces {}",
            vf.canonical_bytes_length, CANONICAL_BYTES_LEN
        );
        return ExitCode::from(2);
    }

    println!("sworn-vector-check: SPEC v0.1-final ({} vectors)", vf.vectors.len());
    println!();

    let reports: Vec<VectorReport> = vf.vectors.iter().map(run_vector).collect();
    let mut passed = 0;
    let mut failed = 0;

    for r in &reports {
        // Per-vector diagnostics: canonical / signature / verify separately.
        let overall = if r.passed() { "✓" } else { "✗" };
        println!(
            "  {} {:<40}  canonical {}  signature {}  verify {}",
            overall,
            r.name,
            VectorReport::status_glyph(r.canonical_ok),
            VectorReport::status_glyph(r.signature_ok),
            VectorReport::status_glyph(r.verify_ok),
        );
        for detail in &r.details {
            for line in detail.lines() {
                println!("      {}", line);
            }
        }
        if r.passed() {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    println!();
    println!("{}/{} passed.", passed, reports.len());

    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
