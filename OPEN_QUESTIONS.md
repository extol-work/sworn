# SWORN — Open Questions

Items deferred from v0.1 that need decisions before their respective future versions. Tracked here so we don't lose them.

## Deferred to v0.2

### Two-layer witness/certifier roles
A pattern where an *entity* (organization) can endorse a *persona* (individual signer) and sign entity-binding statements. Collapse rule when person and entity are the same.

**Why deferred:** Umbriel's primer treats this as central; Ken pushed back that this is a use case pattern, not a core protocol. v0.1 has one signer type. If v0.2 adds roles, this should be a companion spec (`SWORN-Extensions-Roles.md`), not a change to core.

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

### Q1 — Storage of the off-chain payload
Currently spec says "implementation-defined." Is that too loose? Should we at least specify that the storage location must be discoverable from the on-chain record (via a `data_uri` field or convention)?

**Working position:** yes, define `data_uri` as an optional field on the attestation record. Implementations MAY store payloads inline in the notarization substrate (small payloads) or MAY use `data_uri` to reference off-chain storage. Verifiers MUST be able to reach the payload via the URI if present, but MUST accept attestations without a `data_uri` as valid at the metadata level.

### Q2 — Timestamp source
The attestation record includes a timestamp. Is it signer-asserted (self-reported), notarization-asserted (from the block/commit time), or both?

**Working position:** both. Signer-asserted timestamp captures the moment of signing; notarization timestamp captures the moment of commitment. Verifiers should present both, distinctly.

### Q3 — Activity type namespace policy
URI-based extension is clean but URI-squatting is a problem. Do we need a registration process, or is first-writer-wins acceptable?

**Working position:** first-writer-wins with a light registry (JSON file in this repo) for discoverability. If two implementations claim the same URI and disagree on the schema, both are technically conforming; the graph will sort it out because verifiers can inspect the schema referenced by the URI.

### Q4 — Retention hint enforceability
`retention_hint` is metadata the signer expresses ("I intend this to last N days"), but the on-chain notarization is durable regardless. Do we enforce anything, or is it purely advisory?

**Working position:** advisory in v0.1. Implementations MAY offer "close attestation" flows that mark records as expired past the retention hint, but the on-chain hash remains durable. This is a UX / rent-recovery convenience, not a protocol invariant.

### Q5 — Signature algorithm agility
Ed25519 mandatory-to-implement is clean today. If post-quantum signature schemes become urgent, how does the spec migrate without breaking existing attestations?

**Working position:** signature algorithm is a field on the attestation record. New algorithms are added to the registry (§9.2) without invalidating existing Ed25519 attestations. Migration is per-signer, per-attestation.

## Questions we've deliberately closed

### √s voting or any voting weight function in spec
**Resolved:** out of scope. Voting math is implementation choice. See CHARON_COMMENTS.md §7 in the strategy folder for the full argument.

### Blockchain vs. Postgres for the notarization substrate
**Resolved:** substrate-agnostic. Both bindings ship as appendices.

### Roles (witness/certifier) in v0.1
**Resolved:** deferred (see above). Single signer type in v0.1.
