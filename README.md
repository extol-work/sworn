# SWORN

**S**igned<br>
**W**itnessed<br>
**O**pen<br>
**R**ecorded<br>
**N**on-transferable<br>

A specification for portable attestations. Trust between builders, expressed as signatures readable by anyone.

## Status

**v0.1, Request for Comment.** Under review. Versions advance by accumulated attestation, not by committee vote.

This is an early specification. It is deliberately incomplete in some places and deliberately opinionated in others. See [RFC.md](./RFC.md) for what we're asking reviewers to weigh in on.

## What SWORN is

A specification for how one party can testify to a fact (that they observed, endorse, or corroborate something) and have that testimony be independently verifiable by anyone, without requiring the verifier to trust the platform where the testimony was originally recorded.

**Five commitments:**

1. **Signed:** Each attestation is a cryptographic signature by a persistent public key.
2. **Witnessed:** An attestation names *who signed* and *what they attest to*. Standing lives with the persistent key.
3. **Open:** Verification is public and requires no permissioned access.
4. **Recorded:** The hash of the attestation payload is committed to a public ledger.
5. **Non-transferable:** Attestations cannot be traded or wrapped in fungible tokens.

## What SWORN is not

A ranking system. A credential registry. A blockchain. A resume. A reputation score. A voting mechanism.

SWORN defines *how* testimony is captured and verified. What you build on top of that (governance, gating, weighting, display) is your implementation's business.

## For reviewers

- Read [RFC.md](./RFC.md) for what we're asking you to weigh in on.
- Try [sworn-postgres](https://github.com/extol-work/sworn-postgres) to sign your first attestation in five minutes.
- Attest to this specification at the commit hash you reviewed. That signature becomes part of the graph.

## Layers

| Layer | Concern | Status in v0.1 |
|---|---|---|
| 1 | Testimony structure and vocabulary | Required |
| 2 | Signing scheme and canonical bytes | Required |
| 3 | Signer identity and registry | Required (single signer type) |
| 4 | Notarization, anchor to public ledger | Required (substrate-agnostic) |
| 5 | Verification and disclosure endpoints | Required |

Product mechanics (token issuance, governance weight functions, voting math, service tiers) are **out of scope** for the specification. Implementations are free to build these on top. Where an implementation converts attestation-derived standing into transferable value, the conversion function MUST be publicly documented (see §1.5, non-transferability firewall).

## Reference implementation

[extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres). Postgres + Ed25519. No blockchain required. Ships alongside this specification.

## Prior art

SWORN inherits from several traditions and improves on them in specific
ways rather than reinventing the ground:

- **W3C Verifiable Credentials** established the model of cryptographically-
  signed claims about a subject, expressed in a portable data format.
  SWORN's Layer 1 borrows this shape. Where VCs assume issuer authority
  hierarchies, SWORN treats every signer as equal-standing and derives
  weight from graph structure at read-time.
- **Open Badges** (IMS Global) built the peer-attested credential pattern
  at scale in education. Its lesson (that portability matters more than
  format) shapes SWORN's substrate-agnostic Layer 4.
- **SoulBound Tokens** and Weyl/Ohlhaver/Buterin's "Decentralized Society"
  established non-transferability as a design constraint for reputation.
  SWORN's Layer 1 makes this normative and adds the transparency
  requirement for standing conversion.
- **Solana Attestation Service (SAS)** is the notarization substrate
  Extol uses today. Appendix A documents that binding; other bindings
  (Postgres + SHA256, git commit hashes) are equally conformant.
- **Elinor Ostrom's commons work** is the reasoning tradition behind
  SWORN's disclosure discipline. Trust is a commons; readable-but-
  unpullable records are how the commons stays governable.
- **The Research Software Engineering (RSE) community** built its own
  attribution infrastructure in the absence of institutional support:
  JOSS reviews software as a publishable artifact, CITATION.cff pins
  citation metadata to source repositories, and CRediT
  (ANSI/NISO Z39.104-2022) provides fourteen contributor roles that
  make it possible to credit the many kinds of work behind a research
  output. SWORN registers the CRediT namespace directly (§9.1.1) so
  that attestations to research contributions can share this vocabulary
  without reinventing it. CITATION.cff files and DOIs are natural
  subjects for SWORN attestations (as content hashes and resolvable
  identifiers, respectively); implementations wanting to speak fluently
  to the research community should treat them as first-class inputs.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). During the RFC period, we especially want:

- Corner cases the layer boundaries don't handle well
- Vocabulary that reads as jargon-heavy to a non-crypto audience
- Missing constraints that would let an adversary defeat the "non-transferable" property
- Implementations that expose spec ambiguity

## License

Apache 2.0. See [LICENSE](./LICENSE).

The SWORN name and logo are trademarks of Extol, Inc. and are not covered by the Apache license. See [TRADEMARKS.md](./TRADEMARKS.md) for what you can do without asking and what requires permission.

## Provenance

Authored by Extol, Inc. as its first public specification. Reviewer signatures accumulate in the SWORN attestation graph itself. See [RFC.md §Attesting to the spec](./RFC.md#attesting-to-the-spec).
