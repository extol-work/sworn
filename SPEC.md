# SWORN Specification v0.1 (Draft — Outline Only)

**Status:** DRAFT. Section headings only. Normative text lands per the schedule in [ROADMAP.md](./ROADMAP.md).

**Notation:** The words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are used per RFC 2119 conventions.

---

## §1 Overview

### §1.1 Purpose
### §1.2 Terminology
### §1.3 Layer model
### §1.4 Notational conventions
### §1.5 Non-transferability firewall (normative)

> Attestations MUST NOT be transferable between keys. Implementations MUST NOT wrap attestations in fungible tokens. Any conversion of attestation-derived standing into transferable value (governance weight, token allocations, service tiers) MUST be publicly documented, versioned, and referenceable by URI. Documentation MUST specify: (a) inputs from the attestation graph, (b) output value form, (c) any subjective or off-chain inputs, (d) effective date of the current version.

---

## §2 Layer 1 — Testimony

### §2.1 Attestation record structure
### §2.2 Activity type namespacing (URI-based extension mechanism)
### §2.3 Canonical JSON encoding for the semantic payload
### §2.4 activity_hash and data_hash primitives
### §2.5 Subject and witness_for fields
### §2.6 Timestamp and retention_hint

---

## §3 Layer 2 — Signing

### §3.1 Canonical byte sequence for signing
### §3.2 Ed25519 (mandatory-to-implement)
### §3.3 Signature algorithm extension mechanism
### §3.4 Replay protection (nonce derivation)
### §3.5 Key rotation considerations (non-normative, implementation guidance)

---

## §4 Layer 3 — Registry

### §4.1 Signer identity model (single signer type, v0.1)
### §4.2 Persistent key semantics
### §4.3 Revocation by additive attestation
### §4.4 Standing as an emergent property of the graph
### §4.5 What is NOT in this layer (roles, affiliation, delegation — reserved for future)

---

## §5 Layer 4 — Notarization

### §5.1 Hash-anchor commitment (substrate-agnostic)
### §5.2 Merkle batching
### §5.3 Retention semantics (per-record, differing retention allowed)
### §5.4 Durability guarantees for the on-chain hash
### §5.5 Off-chain payload storage (implementation-defined)
### §5.6 What is NOT in this layer (specific chain choice, PDA layouts — see appendices)

---

## §6 Layer 5 — Presentation

### §6.1 Verification endpoint contract
### §6.2 Two-call design (verify metadata / disclose payload with subject consent)
### §6.3 Disclosure token semantics
### §6.4 Refused operations (list-by-subject, bulk export, name search)
### §6.5 Rate limiting and abuse considerations

---

## §7 Security considerations

### §7.1 Sybil resistance (bounded, not absolute)
### §7.2 Attack cost model
### §7.3 Colluding attestation rings — graph-analysis detection
### §7.4 Key compromise and revocation
### §7.5 Payload availability vs. hash durability

---

## §8 Privacy considerations

### §8.1 Public verification, private payloads
### §8.2 Subject-mediated disclosure
### §8.3 Right to be forgotten and immutable hashes
### §8.4 Pseudonymity of witnesses

---

## §9 IANA-style registries (or equivalent for a young spec)

### §9.1 Activity type namespace registry
### §9.2 Signature algorithm registry
### §9.3 Notarization substrate registry

---

## §10 Conformance

### §10.1 Conformance levels
### §10.2 Interoperability tests
### §10.3 Registration process (or self-declaration during RFC period)

---

## Appendix A — Solana / SAS binding (informative)

Extol's on-chain implementation notes. First production adopter. Not normative.

## Appendix B — Postgres binding (informative)

Reference implementation notes. See [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres).

## Appendix C — Worked examples

- Individual attestation (single subject, single signer)
- Batched attestation (Merkle-root, many subjects)
- Additive revocation
- Cross-implementation verification (Postgres signer verified by Solana implementation)

## Appendix D — Glossary

## Appendix E — Changelog
