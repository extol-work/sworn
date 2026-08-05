# SWORN v0.1-final Reference Test Vectors

**Purpose:** cross-implementation identity anchor per [SPEC §10.4](../../../SPEC.md#§10.4-reference-test-vectors). Any conforming SWORN implementation MUST reproduce every `expected_canonical_bytes_hex` and `expected_signature_hex` byte-for-byte given the corresponding `input_fields` and `signer_secret_seed_hex`.

## Format

`vectors.json` is a single JSON file containing an array of vectors under the `vectors` key. Each vector has:

| Field | Meaning |
|---|---|
| `name` | short slug identifying the vector |
| `notes` | prose describing what edge case the vector exercises |
| `spec_version` | always 2 for v0.1-final |
| `input_fields` | all thirteen fields of `AttestationFields`, with 32-byte fields as lowercase hex |
| `signer_secret_seed_hex` | the 32-byte Ed25519 seed used to sign; deterministic so the vector is reproducible |
| `expected_canonical_bytes_hex` | the 248-byte canonical byte sequence per §3.1, as lowercase hex (496 chars) |
| `expected_canonical_bytes_len` | always 248 for v0.1-final |
| `expected_signature_hex` | the 64-byte Ed25519 signature over the canonical bytes, as lowercase hex (128 chars) |

Ed25519 signatures under RFC 8032 PureEdDSA are deterministic functions of `(seed, message)`, so no randomness is involved and every reproduction of a vector produces the same signature bytes.

## Current vectors

Five vectors covering the primary shapes:

1. **`orcid_authorship_happy_path`**: every field non-zero. ORCID-sourced, computed match, high confidence, self-attested. Baseline case.
2. **`self_reported_sourceless`**: zero `source_hash` and `witness_for`; `source_type = SelfReported (1)`. Exercises the zero-source-hash requirement from spec §2.4.
3. **`peer_witnessed_physical`**: highest-trust witnessing pattern. `source_type = PeerWitnessed (9)`, `witnessing_depth = PhysicallyObserved (1)`, `attestor_relationship = Peer (3)`.
4. **`backfilled_migration`**: represents a v0.1-preview row backfilled with best-effort provenance during migration. Legitimate v0.1-final signature with low confidence and `Unspecified` witnessing depth.
5. **`all_zero_provenance_edge_case`**: every provenance field at its zero value. `source_type = Unknown (0)`, `witnessing_depth = Unspecified (0)`, `attestor_relationship = Unknown (0)`, `confidence = 0`.

## Reproducing

The vectors are emitted from [`sworn-postgres/verify/examples/emit_vectors.rs`](https://github.com/extol-work/sworn-postgres/blob/main/verify/examples/emit_vectors.rs). To reproduce locally:

```
git clone https://github.com/extol-work/sworn-postgres
cd sworn-postgres
cargo run -p sworn-verify --example emit_vectors > /tmp/vectors.json
diff /tmp/vectors.json /path/to/sworn/fixtures/attestations/v0.1-final/vectors.json
```

An empty diff means your build reproduces the vectors byte-for-byte.

## Verifying against your implementation

The recommended pattern is a test harness that iterates every vector and, for each:

1. Reconstruct `AttestationFields` from `input_fields`.
2. Compute your implementation's canonical bytes from those fields.
3. Assert equal to `expected_canonical_bytes_hex`.
4. Derive an Ed25519 signing key from `signer_secret_seed_hex`.
5. Sign the canonical bytes.
6. Assert equal to `expected_signature_hex`.

Any assertion failure indicates a bug in either serialization, canonicalization, or signing. The diff is byte-visible.

## Contributing new vectors

Implementations that discover new edge cases in production use SHOULD contribute additional vectors. Open a PR adding to `emit_vectors.rs` and regenerate `vectors.json`. Vectors SHOULD be deterministic (fixed seed bytes) and SHOULD document what edge case they exercise in the `notes` field.

## What these vectors do NOT test

Reference vectors verify canonical byte layout and signature computation. They do not test:

- Payload canonicalization (see spec §2.3 / RFC 8785); implementations should have separate JSON canonicalization tests
- Source hash canonicalization per source_type (see spec §9.2); cross-implementation source-identity vectors are a future addition
- Notarization substrate behavior (Layer 4)
- Presentation endpoint contract (Layer 5)

Each of those merits its own vector set once the corresponding spec sections have required text.
