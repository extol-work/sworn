# SWORN v0.1-final vector-check runner (Rust)

Independent Rust implementation of the SWORN canonical byte sequence and
Ed25519 signing path. Reads the golden vectors in
`fixtures/attestations/v0.1-final/vectors.json` and verifies each byte-for-byte
against the spec.

**Deliberately self-contained.** No dependency on the [sworn-postgres][sp]
reference implementation. Anyone reviewing the SWORN spec should be able to
read `src/main.rs` alongside SPEC §3.1 and §3.2 and confirm the two agree,
without having to trace through the reference implementation.

[sp]: https://github.com/extol-work/sworn-postgres

Companion to [`../node/`](../node/), the same runner in TypeScript. Running
both against the same vectors and getting matching results confirms that
SWORN's byte-level serialization is unambiguous enough for independent
implementations to agree.

## Usage

```bash
# from this directory
cargo run --quiet -- ../../attestations/v0.1-final/vectors.json

# from anywhere, once installed
cargo install --path .
sworn-vector-check /path/to/vectors.json
```

The path argument is required. There is no default. A runner that silently
checks the wrong vectors is worse than one that refuses to run.

## Expected output

```
sworn-vector-check: SPEC v0.1-final (5 vectors)

  ✓ orcid_authorship_happy_path               canonical ✓  signature ✓  verify ✓
  ✓ self_reported_sourceless                  canonical ✓  signature ✓  verify ✓
  ✓ peer_witnessed_physical                   canonical ✓  signature ✓  verify ✓
  ✓ backfilled_migration                      canonical ✓  signature ✓  verify ✓
  ✓ all_zero_provenance_edge_case             canonical ✓  signature ✓  verify ✓

5/5 passed.
```

Exits `0` if every vector passes all three checks. Exits `1` with per-vector
diagnostics otherwise. Exits `2` on argv, file, or vector-file schema errors.

## Per-vector diagnostics

Each vector is reported against three independent checks:

| Check         | What it proves                                                    | If it fails you have                              |
|---------------|--------------------------------------------------------------------|---------------------------------------------------|
| `canonical`   | Reconstructed 248-byte canonical bytes match `expected_canonical_bytes_hex` | A serialization bug: field order, endianness, or a missing/extra field |
| `signature`   | Signing the canonical bytes with `signer_secret_seed_hex` reproduces `expected_signature_hex` | A signing-path bug: PureEdDSA vs Ed25519ph, seed vs expanded key, wrong nonce derivation |
| `verify`      | `expected_signature_hex` verifies against the reconstructed canonical bytes and the signer pubkey | A verifier-side pubkey handling bug, or transcription error in the vector itself |

Each check is reported separately so a failure diagnoses itself. A canonical
mismatch and a signature mismatch have distinct root causes and are worth
distinguishing at a glance.

## Ed25519 note

The signing path uses `ed25519_dalek::SigningKey::sign` which is PureEdDSA
per RFC 8032 §5.1. **Implementations MUST NOT use Ed25519ph** (SPEC §3.2).
Prehashing changes the signature semantics and produces bytes that will not
verify against SWORN vectors. If you're porting to another language and
your first attempt gives a `signature` mismatch on every vector, check
whether your Ed25519 library is defaulting to Ed25519ph.

Cross-language reference: Node.js's `crypto.sign(null, message, keyObject)`
(with `null` for hash algorithm) is the PureEdDSA path. See [`../node/`](../node/).

## Build requirements

- Rust 2021 edition
- Two dependencies: `ed25519-dalek 2.x` (crypto) and `serde` + `serde_json`
  (vector parsing). No workspace, no build script, no `unsafe`.

## When vectors drift

If this runner fails against a vector file that used to pass, one of two
things has happened:

1. The vector file was regenerated (a spec change or a new edge case).
   Look at the diff to see which vectors changed and why.
2. The runner is being pointed at a vector file for a different spec
   version. The runner rejects `spec_version != 2` at load time with a
   clear error; it will not silently attempt to verify against a mismatched
   canonical byte layout.

Deliberate: silent success against the wrong spec is worse than loud
failure.
