# SWORN v0.1 — Request for Comment

## The problem

Trust between builders — the small, corroborable kind that used to live in guilds, unions, and professional societies — has no portable representation on the modern internet. Every platform reinvents a walled version: LinkedIn endorsements, GitHub stars, Stack Overflow reputation, Discord roles. Each one is real, none is portable, and all of them silently expire when the platform decides to change hands, change policy, or die. The gap keeps builders locked into whichever platform they earned their standing on, and it keeps readers of that standing — employers, funders, collaborators — unable to distinguish real corroboration from platform theater.

## The principle

Trust is established by corroborating layers, not single endorsements. A signature by one person is a claim; a signature by many, each with their own standing at stake, is evidence. Any system that hopes to be trustworthy over decades has to make the corroboration itself durable, portable, and independently verifiable — separate from any single vendor's continued existence.

SWORN is a specification, not a product. It says nothing about how you rank people, how you weight votes, or how you decide who to trust. It specifies only how testimony is *captured* so that anyone can verify it *later*, from anywhere, without needing to trust the platform where it was originally recorded.

## This is a request for comment — and for attestation

This document defines version 0.1 of the SWORN specification. It is deliberately incomplete in some places (multi-signer patterns, cross-chain notarization, delegation) and deliberately opinionated in others (non-transferability, single signer type, substrate-agnostic notarization).

We are not asking for consensus. We are asking for **signed reactions**, using the mechanism the spec defines. Reviewers who agree with the spec, disagree with a section, or want to propose an amendment can express that as a SWORN attestation whose subject is a specific commit hash or PR reference in this repository. See [§Attesting to the spec](#attesting-to-the-spec) below.

The double meaning is intentional. If SWORN is right about how testimony should work, the first useful attestations in the graph should be about the spec itself.

## Scope

**In scope for v0.1:**

- Attestation record structure (subject, signer, hash, timestamp, retention hint)
- Ed25519 signing scheme and canonical byte serialization
- Single signer type (persona-anchored public key)
- Hash-anchor commitment to a public ledger (substrate-agnostic; example bindings for Solana/SAS and Postgres+SHA256 are appendices)
- Merkle batching for large-cardinality events
- Two-call verification API (metadata verification without payload access; payload disclosure with subject consent)
- Non-transferability constraint (attestations cannot be traded; standing conversions must be documented)

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

Authored by Charon (systems engineering), Extol, Inc. Early review from Ken (founder), Umbriel (positioning), Ariel (economic firewall design), and Titania (implementation review). The reference implementation ([extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres)) ships alongside this document.

## Attesting to the spec

To attest to this specification — to endorse it, disagree with a section, or propose an amendment — publish a SWORN attestation using any conforming implementation. The subject of your attestation should be one of:

- The git commit hash of the version you are attesting to (`git rev-parse HEAD` from a clone of this repo)
- A permalink URL to a specific commit or PR (for endorsements of a proposed change)
- The SHA256 of the specific document at the version you reviewed (self-verifying, GitHub-independent)

Attestations to the spec become part of the SWORN graph and are themselves verifiable via any conforming implementation. There is no ratification process; there is only accumulating corroboration.

If you want your attestation to be visible in the RFC review discussion, open a PR referencing it. Do not submit textual endorsements — submit signatures.
