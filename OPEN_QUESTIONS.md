# Open Questions

Items not resolved in normative text. Tracked here so the design questions do not get lost between versions.

## Open in v0.2

### Q1, Payload discoverability

SPEC.md §5.5 says payloads live off-chain and the specification does not require a specific storage model. In practice, verifiers who obtain an attestation may not know where the payload is stored.

**Working position:** implementations MAY publish payload storage locations as off-chain metadata alongside the attestation record. A future version may define an optional `data_uri` field on the record, or a companion metadata document format, once at least two implementations have converged on a shape. Until then, payload discoverability is implementation-defined and belongs in application-layer conventions.

### Q2, Activity type namespace policy

SPEC.md §2.2 permits any well-formed URI as an activity type. URI-squatting is a possible concern in adversarial deployments.

**Working position:** first-writer-wins with a light registry (§9.1 in SPEC.md) for discoverability. Two implementations claiming the same URI and disagreeing on the schema are both technically conforming; verifiers can inspect the schema referenced by the URI and choose which to trust. A future version may add stricter registration guidance if squatting becomes a real problem in the wild.

### Q3, Signature algorithm agility

Ed25519 is mandatory-to-implement. SPEC.md §3.3 permits future versions to register additional algorithms via §9.5.

**Working position:** a future version will define a signature algorithm registry entry (a small identifier appended to canonical bytes or expressed via `spec_version`) that lets attestations carry non-Ed25519 signatures without breaking existing Ed25519 attestations. Migration would be per-signer, per-attestation. The concrete mechanism is deferred until a specific algorithm (post-quantum, threshold, ES256 for WebAuthn) has been identified as a required addition.

### Q4, Key rotation

SPEC.md §3.5 (non-normative) says rotation is expensive: either publish a signed statement from the old key delegating to a new key, or accept that standing does not transfer. Neither path is normatively defined.

**Working position (v0.2):** signatures under a rotated-away key remain cryptographically valid. Revocation under §4.3's additive-attestation model requires the original signing key: only the original signer can publish a retraction. Key rotation as a first-class protocol feature is deferred.

**Deferred to a future version:** a rotation attestation binding a successor key to a predecessor key, allowing the successor to publish revocations against predecessor-signed attestations. Needs its own trust model (the rotation attestation must itself be signed by the predecessor, or by a designated recovery key). Should co-design with Q5.

### Q5, Key compromise

Distinct from rotation. If a signing key is compromised, an attacker can produce valid signatures. The specification does not currently provide a way for a signer to declare "my key was compromised after date X."

**Working position (v0.2):** no protocol-level compromise declaration. Signers whose keys are compromised should publish an out-of-band statement and, when Q4 is defined, a rotation attestation. Verifiers presenting standing MAY consult external compromise registries; the specification does not constrain how.

**Deferred:** a compromise attestation with a claimed compromise timestamp and verifier guidance for weighting attestations signed by that key after the claimed date. Should be co-designed with Q4.

### Q6, Multi-signer entity keys

An earlier version considered a two-layer pattern where an entity (organization) endorses a persona (individual signer) and can sign entity-binding statements distinct from persona-signed statements. This requires role primitives the current specification omits.

**Working position:** this belongs in a companion specification, not in a change to core. A future companion may define entity keys, delegation attestations, and affiliation semantics on top of the existing single-signer-type primitive.

**Prerequisite:** at least one implementation attempts the pattern in the wild and reports what actually broke.

### Q7, Cross-substrate portability

Bindings/sas.md is the only Layer 4 binding at v0.2. If additional Layer 4 bindings emerge (a different chain, a certificate transparency log, a git-anchored append-only log), a verifier holding an attestation whose hash is anchored on binding A may want cryptographic proof that the same hash was also anchored on binding B.

**Working position:** out of scope for v0.2. Cross-substrate proof formats are premature to specify with only one Layer 4 binding in production. Revisit if a second binding is proposed.

### Q8, Merkle batching format

Some notary deployments batch attestation hashes into a Merkle tree and publish only the root. Bindings/sas.md §7 describes batching informally; the specification does not define a normative Merkle format.

**Working position:** deferred to a future version. Waiting until at least two independent implementations need normative batching interoperability produces better format than committing early.

## Closed by v0.2

### Timestamp source and trust posture

**Resolved.** Substrate time is authoritative for time-sensitive claims per SPEC.md §2.7 and §5.1. The signer-claimed timestamp (`signer_asserted_at`) is captured as the signer's assertion but MUST NOT be relied upon by verifiers for time-sensitive trust decisions. Prior working position promoted to normative text.

### Canonical serialization of the payload

**Resolved.** SPEC.md §2.3 requires RFC 8785 (JSON Canonicalization Scheme). Activity type schemas MAY define alternate canonical forms for non-JSON payloads, but the activity type must document the canonicalization rule.

### Retention hint semantics

**Resolved.** SPEC.md §5.3 defines retention as advisory, per-attestation, and explicitly distinct from validity. An attestation whose payload has been reclaimed remains valid at Layer 2; only its disclosability changes.

### Self-attestation

**Resolved.** Self-attestation is permitted at the protocol layer. Graph-analysis interpretation (how to weight self-attestations relative to peer attestations) is a reader-side concern, out of scope for the specification.

### Voting math

**Resolved.** Voting weight functions are out of scope. Applications compute their own weights from the attestation graph and are subject to their own transparency requirements.

### Notarization substrate

**Resolved.** Solana Attestation Service is the required Layer 4 binding for full conformance. Other substrates may satisfy Layers 1 and 2 only (see bindings/postgres.md).

### Payload on the notary substrate

**Resolved.** Payloads are not stored on the notary substrate. The substrate commits to the payload hash only. Implementations MAY store payloads in a separate substrate at their own cost; this is off the specification's path.

### Non-transferability firewall

**Resolved.** The former §1.5 firewall (mandating transparency around standing-to-value conversions) is retired. Non-transferability is not enforceable at Layer 1 and belongs in application policy. See PRIMER.md for the reasoning.

### Witnessing as a protocol operation

**Resolved.** Witnessing is not a protocol operation. The `witness_for` field is a pointer, not an operation. Multi-party witnessing is composed by applications as pairs of independent attestations.
