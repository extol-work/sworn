# SWORN v0.1: Request for Comment

**S**igned, **W**itnessed, **O**pen, **R**ecorded, **N**on-transferable.
A specification for durable, portable testimony.

## The problem

AI has made every unverified signal fakeable. The signals that survive are the ones a specific human staked their name on.

Trust between builders, the small and simple to corroborate kind that used to live in guilds, unions, and professional societies, has no portable representation on the modern internet. Every platform reinvents a walled version: LinkedIn endorsements, GitHub stars, Stack Overflow reputation, Discord roles. Each one is real, none is portable, and all of them silently expire when the platform decides to change hands, change policy, or die. The gap keeps builders locked into whichever platform they earned their standing on. It keeps readers of that standing (employers, funders, collaborators) unable to distinguish real corroboration from platform theater.

## The principle

Trust is established by corroborating layers, not single endorsements or an account balance. A signature by one person is a claim. A signature by many, each with their own standing at stake, is evidence. Any system that hopes to be trustworthy over decades has to make the corroboration itself durable, portable, and independently verifiable, separate from any single vendor's continued existence.

SWORN is a specification, not a product. It says nothing about how you rank people, how you weight votes, or how you decide who to trust. It specifies only how testimony is *captured* so that anyone can verify it *later*, from anywhere, without needing to trust the platform where it was originally recorded.

## This is a request for comment, and for attestation

This document defines version 0.1 of the SWORN specification. It is deliberately incomplete in some places (multi-signer patterns, cross-chain notarization, delegation) and deliberately opinionated in others (non-transferability, single signer type, substrate-agnostic notarization).

We are not asking for consensus. We are asking for **signed reactions**, using the mechanism the spec defines. Reviewers who agree with the spec, disagree with a section, or want to propose an amendment can express that as a SWORN attestation whose subject is a specific commit hash in this repository.

**There is no ratification process. There is only accumulating corroboration.**

The spec proves itself through its own use. If SWORN is right about how testimony should work, the first useful attestations in the graph should be about the spec itself. See [§Attesting to the spec](#attesting-to-the-spec) below for the practical path.

## Scope

**In scope for v0.1:**

- Attestation record structure (subject, signer, hash, timestamp, retention hint)
- Ed25519 signing scheme and canonical byte serialization
- Single signer type (persona-anchored public key)
- Hash-anchor commitment to a public ledger. Substrate-agnostic. Example bindings for Solana/SAS and Postgres/SHA256 are appendices.
- Merkle batching for large-cardinality events
- Two-call verification API: metadata verification without payload access, and payload disclosure with subject consent.
- **Non-transferability.** Attestations MUST NOT be transferable between keys.
- **Standing-conversion transparency.** Any conversion of attestation-derived standing into other value forms (governance weight, token allocations, service tiers) MUST be publicly documented and versioned.

**Deliberately out of scope for v0.1:**

- Two-layer witness/certifier role patterns (candidates for a companion spec)
- Entity attestations and affiliation revocation
- Voting weight functions or governance math (implementation choice)
- Token issuance patterns or economic mechanisms
- Cross-chain notarization proofs
- Voter delegation
- Multi-signature attestation patterns
- Zero-knowledge disclosure

Deferred items are enumerated in [OPEN_QUESTIONS.md](./OPEN_QUESTIONS.md) so future versions can address them without re-litigating the core.

## Provenance

Authored by Extol, Inc. as its first public specification. Reviewer signatures accumulate in the SWORN attestation graph itself.

The reference implementation ([extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres)) ships alongside this document. The intellectual lineage (the historical framework for why witnessed testimony matters, the institutions that failed to build it durably, and the design principles that shape SWORN's constraints) is developed in the Extol canon (link to be added when the canon publishes, soon).

## Attesting to the spec

Attestations to the spec become part of the SWORN graph and are themselves verifiable by any conforming implementation.

**Path 1: direct signature.** The reference implementation hosts a public collection endpoint at [sworn.extol.work/rfc](https://sworn.extol.work/rfc) *(coming with the sworn-postgres v0.1 release)*. Submit a signed attestation whose subject is a commit hash of this repository. The endpoint verifies your signature, records the attestation, and returns a permalink. You do not need a GitHub account.

**Path 2: five-minute quickstart.** If you want to generate a keypair and sign your first attestation from scratch, follow [sworn-postgres README](https://github.com/extol-work/sworn-postgres#quickstart). The RFC is the recommended first subject.

**Path 3: PR-referenced attestation.** If you want your attestation visible in the GitHub review discussion, submit via Path 1, then open a PR that references the permalink. This is optional and adds no cryptographic weight, only social visibility.

Valid subjects for an attestation to this spec:

- The git commit hash of the version you are reviewing (`git rev-parse HEAD` from a clone of this repo)
- A permalink URL to a specific commit or PR (for endorsements of a proposed change)
- The SHA256 of the specific document at the version you reviewed (self-verifying, GitHub-independent)

Do not submit textual endorsements. Submit signatures.
