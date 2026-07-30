# SWORN, Open Questions

Items deferred from v0.1 that need decisions before their respective future versions. Tracked here so we don't lose them.

## Deferred to v0.2

### Two-layer witness/certifier roles
A pattern where an *entity* (organization) can endorse a *persona* (individual signer) and sign entity-binding statements. Collapse rule when person and entity are the same.

**Why deferred:** Umbriel's primer treats this as central. Ken pushed back that this is a use case pattern, not a core protocol. v0.1 has one signer type. If v0.2 adds roles, this should be a companion spec (`SWORN-Extensions-Roles.md`), not a change to core.

**Prerequisite:** at least one implementation attempts the pattern in the wild and reports what actually broke.

### Affiliation revocation semantics
Distinct from attestation revocation. When a persona leaves an entity, past attestations should remain valid but the affiliation itself should be marked as ended, without allowing the entity to retroactively invalidate the persona's testimony.

**Why deferred:** requires the roles model above.

### Entity-binding statements
A signature by an entity that references the entity's persistent identity, not any specific member. E.g., a validator organization signing its own block-building disclosure.

**Why deferred:** requires the roles model above.

## Deferred to v0.3+

### Voter delegation
A signer delegating attestation-derived weight to another signer for a specific decision or period.

### Multi-signature attestations
An attestation requiring N-of-M signatures before it becomes valid. Distinct from independent attestations by multiple signers about the same subject.

### Zero-knowledge disclosure
Proving properties of an attestation set (e.g., "signer has ≥3 attestations of type X") without revealing which specific attestations.

### Cross-chain notarization proofs
Standardized proof that a hash committed on chain A is the same as one committed on chain B, for portability across notarization substrates.

## Open questions inside v0.1 (must resolve before ratification)

### Q1, Storage and availability of the off-chain payload
Currently spec says "implementation-defined." Is that too loose? Should we at least specify that the storage location must be discoverable from the on-chain record (via a `data_uri` field or convention)?

**Working position:** yes, define `data_uri` as an optional field on the attestation record. Implementations MAY store payloads inline in the notarization substrate (small payloads) or MAY use `data_uri` to reference off-chain storage.

Availability is distinct from validity:

- An attestation is **valid** if the signature verifies against the notarized hash. Validity is a property of the signature and the hash, not the payload.
- An attestation is **disclosable** if the payload is retrievable and re-hashes to the notarized value. Disclosability is a property of the payload's availability at the URI (or inline).

A valid attestation whose payload is temporarily (or permanently) unreachable remains valid. Verifiers MUST NOT treat a dead URI as an invalidity signal. This aligns with the two-call verification model in Layer 5: `GET /verify` operates on the hash alone; `POST /disclose` is where availability matters.

Rationale: otherwise a hostile actor could break attestation validity by taking down a URI they don't control.

### Q2, Timestamp source and trust posture
The attestation record includes a timestamp. Is it signer-asserted (self-reported), notarization-asserted (from the block/commit time), or both?

**Working position:** both are captured, but they carry different trust weight.

- **Signer-asserted timestamp:** captures the moment the signer claims to have signed. Informational only. Can be lied about (a signer can back-date a 2026 claim to 2023 for a tenure application).
- **Notarization timestamp:** captures the moment the hash was committed to the notarization substrate. Cannot be back-dated once the substrate is public.

Verifiers MUST treat the notarization timestamp as the trust-relevant timestamp for any time-sensitive claim (recency, seniority, precedence, cooldowns). Signer-asserted timestamps MAY be presented alongside for context but MUST NOT be relied upon by verifiers for trust decisions.

Rationale: without this constraint, a signer can back-date an entire portfolio the moment they realize the graph is being queried for weight-carrying claims.

### Q3, Activity type namespace policy
URI-based extension is clean but URI-squatting is a problem. Do we need a registration process, or is first-writer-wins acceptable?

**Working position:** first-writer-wins with a light registry (JSON file in this repo) for discoverability. If two implementations claim the same URI and disagree on the schema, both are technically conforming. The graph will sort it out because verifiers can inspect the schema referenced by the URI.

### Q4, Retention hint semantics
`retention_hint` is metadata the signer expresses ("I intend this to last N days"), but the on-chain notarization is durable regardless. Do we enforce anything, or is it purely advisory?

**Working position:** advisory, and framed as a cost/UX convenience rather than a validity property.

Implementations MAY reclaim local storage, notarization rent, or close their own copies past the retention hint. The on-chain hash and any external replicas remain durable regardless of what any single implementation chooses to do. Retention is a hint about the signer's intent and about implementation storage economics, not a signal that the attestation has been withdrawn or invalidated.

Rationale: an implementation that "expires" a record cannot bind other verifiers who kept their own copies. Framing retention as a validity property invites the misinterpretation that expired records are invalid, which they are not.

### Q5, Signature algorithm agility
Ed25519 mandatory-to-implement is clean today. If post-quantum signature schemes become urgent, how does the spec migrate without breaking existing attestations?

**Working position:** signature algorithm is a field on the attestation record. New algorithms are added to the registry (§9.2) without invalidating existing Ed25519 attestations. Migration is per-signer, per-attestation.

### Q6, Key rotation
If a signer rotates from key A to key B, do old signatures under A remain valid? Can key B revoke an attestation signed by key A?

**Working position (v0.1):** old signatures under key A remain cryptographically valid indefinitely. Revocation under the additive-attestation model requires the original signing key: only key A can publish a retraction of an attestation signed by A. Key rotation as a first-class protocol feature is deferred.

**Deferred to v0.2:** a rotation attestation binding key B as a successor to key A, allowing B to publish revocations against A-signed attestations. Needs its own trust model (rotation attestation must itself be signed by A, or by a designated recovery key). Out of v0.1 scope so we don't ship a half-baked recovery story.

### Q7, Key compromise
Distinct from rotation. If a signing key is compromised, an attacker can produce valid signatures. There should be a way for a signer to declare "my key was compromised after date X, discount attestations signed by it after that point."

**Working position (v0.1):** no protocol-level compromise declaration. Signers whose keys are compromised should publish an out-of-band statement (and, if they have a successor key ready, a rotation attestation once Q6 is defined). Verifiers presenting standing MAY consult external compromise registries, but nothing at Layer 1 through 5 constrains how.

**Deferred to v0.2:** a compromise attestation with a claimed compromise timestamp, and verifier guidance for weighting attestations signed by that key after the claimed date. Should be co-designed with Q6 because they share machinery.

### Q8, Canonical serialization of the payload
The spec says the hash is over the canonical form of the payload, but canonicalization is a known hard problem. Two implementations disagreeing on canonicalization produce different hashes for the same logical payload, and neither can verify the other.

**Working position:** for JSON payloads, canonicalization MUST follow RFC 8785 (JSON Canonicalization Scheme). Activity type schemas MAY define alternate canonical forms for non-JSON payloads (CBOR, Protobuf, etc.), but the activity type registry entry MUST specify the canonicalization rule. If unspecified, RFC 8785 is the default.

Rationale: canonicalization is the single most common source of "why don't our hashes match" bugs. Naming a specific rule (rather than "canonical JSON") removes the ambiguity that has bitten every prior attestation-adjacent spec.

### Q9, Self-attestation
Can a signer sign an attestation about themselves? The spec appears to allow it (anyone signs anyone), which is correct, but the graph-analysis implications should be named somewhere.

**Working position:** self-attestation is permitted at the protocol level. It exists as a node in the graph with no external corroboration. Implementations that compute standing or weight from the graph SHOULD treat self-attestations differently from peer attestations (typically by discounting or ignoring them). This is guidance for implementers, not a Layer 1 through 5 constraint.

Rationale: naming this explicitly prevents implementations from being surprised when a signer bulk-publishes self-attestations. The graph-analysis response is well-understood but should be documented.

## Questions we've deliberately closed

### √s voting or any voting weight function in spec
**Resolved:** out of scope. Voting math is implementation choice. See CHARON_COMMENTS.md §7 in the strategy folder for the full argument.

### Blockchain vs. Postgres for the notarization substrate
**Resolved:** substrate-agnostic. Both bindings ship as appendices.

### Roles (witness/certifier) in v0.1
**Resolved:** deferred (see above). Single signer type in v0.1.

### Storing the full payload on-chain
**Resolved:** the notarization substrate commits only the hash of the canonicalized payload. Payloads are never stored on-chain in v0.1. An implementation is free to store full payloads on-chain at its own cost, but doing so is not part of the spec and doesn't affect conformance. Off-chain storage (URI-addressable or otherwise) is the intended pattern.

Rationale: pre-empts the "why don't we just put the JSON on-chain" question a Solana-native reviewer will inevitably ask.
