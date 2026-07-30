# SWORN

**S**igned, **W**itnessed, **O**pen, **R**ecorded, **N**on-transferable.

A specification for portable attestations. Trust between builders, expressed as signatures readable by anyone.

## Status

**v0.1 — Request for Comment.** Under review. Not yet ratified.

This is an early specification. It is deliberately incomplete in some places and deliberately opinionated in others. See [RFC.md](./RFC.md) for what we're asking reviewers to weigh in on.

## What SWORN is

A specification for how one party can testify to a fact — that they observed, endorse, or corroborate something — and have that testimony be independently verifiable by anyone, without requiring the verifier to trust the platform where the testimony was originally recorded.

**Five commitments:**

1. **Signed.** Each attestation is a cryptographic signature by a persistent public key.
2. **Witnessed.** An attestation names *who signed* and *what they attest to*. Standing lives with the persistent key.
3. **Open.** Verification is public and requires no permissioned access.
4. **Recorded.** The hash of the attestation payload is committed to a public ledger.
5. **Non-transferable.** Attestations cannot be traded or wrapped in fungible tokens.

## What SWORN is not

A ranking system. A credential registry. A blockchain. A resume. A reputation score. A voting mechanism.

SWORN defines *how* testimony is captured and verified. What you build on top of that — governance, gating, weighting, display — is your implementation's business.

## Layers

| Layer | Concern | Status in v0.1 |
|---|---|---|
| 1 | Testimony structure and vocabulary | Normative |
| 2 | Signing scheme and canonical bytes | Normative |
| 3 | Signer identity and registry | Normative (single signer type) |
| 4 | Notarization / anchor to public ledger | Normative (substrate-agnostic) |
| 5 | Verification and disclosure endpoints | Normative |

Product mechanics — token issuance, governance weight functions, voting math, service tiers — are **out of scope** for the specification. Implementations are free to build these on top. Where an implementation converts attestation-derived standing into transferable value, the conversion function MUST be publicly documented (see §1.5, non-transferability firewall).

## Reference implementation

[extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres) — Postgres + Ed25519. No blockchain required. Ships alongside this specification.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). During the RFC period, we especially want:

- Corner cases the layer boundaries don't handle well
- Vocabulary that reads as jargon-heavy to a non-crypto audience
- Missing constraints that would let an adversary defeat the "non-transferable" property
- Implementations that expose spec ambiguity

## License

Apache 2.0. See [LICENSE](./LICENSE).

## Provenance

Authored by Extol, Inc. as its first public specification. Contributions from Umbriel (positioning), Ariel (economic firewall), Titania (implementation review), and Charon (systems architecture). Reviewer signatures accumulate in the SWORN attestation graph itself — see [RFC.md §Attesting to the spec](./RFC.md).
