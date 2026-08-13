# SWORN Binding: Postgres (sworn-postgres)

**Status:** Informative. This document describes how the reference Postgres implementation at [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres) satisfies Layers 1 (Testimony) and 2 (Signing) of SPEC.md v0.2. This binding does NOT provide Layer 4 (Notarization) conformance; see §4 below.

**Reader.** Implementers evaluating whether sworn-postgres fits their needs, or implementers building a similar Layer 1+2 emitter in another database, will find this useful. Implementers building a full Layer 1 through Layer 5 conforming implementation should read bindings/sas.md instead, since Layer 4 conformance requires SAS.

---

## §1 What sworn-postgres is

`sworn-postgres` is a Postgres schema plus a small Rust binary that together produce and store SWORN attestations conforming to SPEC.md Layers 1 (Testimony) and 2 (Signing).

Specifically it:

- Constructs the 248-byte canonical byte sequence per SPEC.md §3.1 exactly.
- Signs it with Ed25519 PureEdDSA per SPEC.md §3.2.
- Stores the resulting attestation record in Postgres tables with a schema that preserves every field needed to reconstruct the canonical bytes for later verification.
- Emits reference test vectors per SPEC.md §10.4 that cross-check against the two other reference implementations.

It does NOT:

- Anchor the attestation hash to any public substrate.
- Provide the durability guarantees SPEC.md §5.4 requires of a Layer 4 notary.
- Prevent the operator from silently modifying stored attestations (Postgres is authoritative to whoever holds the credentials).

## §2 What conformance this binding provides

A deployment using sworn-postgres alone satisfies SPEC.md §10.1 Level 2 (Signer):

- Produces byte-for-byte conforming canonical bytes.
- Produces valid Ed25519 signatures.
- Populates provenance fields per SPEC.md §2.5.
- Passes SPEC.md §10.4 reference vectors.

It does NOT satisfy Level 3 (Notarizer), which requires SAS anchoring per bindings/sas.md.

**Practical implication.** A signed attestation produced by sworn-postgres is cryptographically valid: any verifier with the canonical bytes and signature can verify it per SPEC.md §3.1's verification procedure. What the verifier cannot do is confirm the attestation was published to a tamper-evident public substrate at a specific time. That confirmation requires the anchor to have been produced via bindings/sas.md.

## §3 When sworn-postgres is the right choice

sworn-postgres is a reasonable Layer 1+2 emitter for:

- **Local development and testing.** Producing signed attestations for tests without incurring Solana transaction costs.
- **Golden vector generation.** Producing SPEC.md §10.4 reference vectors for cross-implementation validation.
- **Non-notarized signing.** Applications that need signed attestations but do not need or want public notarization. Examples: internal audit trails within a single trusted-boundary organization, ephemeral attestations that are never intended for third-party verification.

sworn-postgres is NOT the right choice for:

- **Third-party-verifiable attestations.** If any party outside your organization needs to trust the attestation exists at a specific time, you need Layer 4. Use bindings/sas.md.
- **Attestations you plan to display as verifiable public evidence.** The same reason.
- **Standing accumulation across implementations.** The interoperability property that makes SWORN's graph portable requires Layer 4 conformance.

## §4 Why this is not a Layer 4 binding

A verifier trusting sworn-postgres's Postgres storage as the substrate is trusting the operator of that Postgres instance. If the operator later rewrites history (updates a stored attestation's fields, deletes a row, changes a timestamp), the verifier has no cryptographic path to detect the change beyond the individual signature.

The signature detects payload tampering (data_hash covers the payload; signature covers data_hash). The signature does NOT detect:

- Silent deletion of an attestation record.
- Silent modification of the `signer_asserted_at` field (which is signed content, so the signature would fail to verify after modification, but the operator can also present the modified attestation without the signature and claim the row was never signed).
- Silent modification of application-layer metadata (retention state, revocation status, disclosure grants) that the specification does not sign but that applications typically rely on.

Layer 4 is what defeats these attacks: a public substrate over which the operator has no more privilege than any other observer. Postgres does not provide that property to third-party verifiers.

**A future migration path.** A deployment that starts on sworn-postgres for Layers 1+2 and later wants Layer 4 conformance can add SAS anchoring by running each attestation through bindings/sas.md's §5 CreateAttestation flow. Legacy attestations that were never anchored remain valid but not Layer 4 conforming; new attestations post-migration are Layer 3 (Notarizer) conforming. sworn-postgres would remain in the picture as the local signing surface, with SAS added as the notary.

## §5 Schema and code

The Postgres schema is at [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres) under `migrations/`. Key tables:

- `attestations`: one row per attestation, one column per field in SPEC.md §3.1's canonical byte sequence, plus the signature and any application-layer metadata.
- `payloads`: separate table for payload bytes, referenced from `attestations` by content hash.

The signing binary is at `src/main.rs`. The verification binary is at `src/verify.rs`. Reference vectors are at `fixtures/vectors/`.

Cross-implementation validation runs sworn-postgres's vector emitter against the vectors expected by Titania's TypeScript runner and the standalone Rust runner in the main SWORN repository's `fixtures/runners/`. All three implementations agree byte-for-byte.

## §6 Relationship to Extol's production deployment

Extol's production deployment does not use sworn-postgres as its signing surface. Extol's signing happens in the passkey-derived client flow described in the [extol-work/extol-cortex](https://github.com/extol-work/extol-cortex) documentation, with SAS as the notary via bindings/sas.md.

sworn-postgres exists as a reference implementation and interoperability anchor, not as a production dependency. Its role is to prove that SWORN's byte layout and signature semantics are reproducible outside Extol's application stack. If sworn-postgres and Extol's client-side implementation ever produced different canonical bytes for the same input, the specification would have failed at Layer 1 or 2.

This is what makes the specification portable rather than a codebase-with-README-labeled-spec: multiple independent implementations agree on the bytes.

---

## Appendix A: Adding a similar Layer 1+2 binding for another database

Implementers who want to build a similar emitter for a different database (SQLite, DuckDB, MongoDB, DynamoDB) can follow sworn-postgres's shape:

1. Schema: one row per attestation, one column per SPEC.md §3.1 field, plus signature.
2. Payload storage: separate table or content-addressed store, keyed by data_hash.
3. Signing: any Ed25519 library that produces PureEdDSA signatures per RFC 8032.
4. Canonicalization: RFC 8785 JSON canonicalization for payloads.
5. Cross-implementation validation: emit SPEC.md §10.4 reference vectors and confirm byte agreement with at least one other conforming implementation.

The database choice does not affect conformance: any database that preserves the fields losslessly and lets the application reconstruct canonical bytes from stored rows works. The Postgres choice for sworn-postgres is convenience, not requirement.
