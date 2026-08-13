# Attestation Notary Specification

**Draft v0.2.** Under review. Not yet accepting external signatures.

A specification for signed factual claims and their notarization to a public substrate. Testimony that a verifier can check without asking the platform of origin.

## What this specification defines

- How one party signs a factual claim about another party or artifact, in canonical bytes any implementation can reproduce.
- What provenance those bytes carry (source, confidence, witnessing depth, attestor relationship).
- How a hash of those bytes is committed to a public notary substrate whose account layout does not expose signer, subject, or content in enumerable form.
- How a third party verifies the signature and, with the signer's authorization, retrieves the payload.

The specification is short (five layers) and deliberately narrow. See [SPEC.md](./SPEC.md) for the normative text.

## What this specification does not define

Witnessing as a protocol operation, non-transferability of attestations, ranking or scoring functions, role and delegation patterns, KYC or identity assurance. These belong to applications built on top. See [PRIMER.md](./PRIMER.md) for the reasoning.

## Layout

| File | Role |
|---|---|
| [SPEC.md](./SPEC.md) | Normative. MUST / MUST NOT / SHOULD language only. |
| [PRIMER.md](./PRIMER.md) | Non-normative. Rationale, philosophy, product context, historical notes. |
| [bindings/sas.md](./bindings/sas.md) | Normative. Solana Attestation Service binding. Required for Layer 4 conformance. |
| [bindings/postgres.md](./bindings/postgres.md) | Informative. Postgres binding via [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres). Layers 1 and 2 only. |
| [fixtures/](./fixtures/) | Golden test vectors and cross-implementation runners. |
| [OPEN_QUESTIONS.md](./OPEN_QUESTIONS.md) | Questions not yet resolved in normative text. |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | How to submit review or an implementation. |

## Layer summary

| Layer | Concern |
|---|---|
| 1: Testimony | Canonical byte layout, provenance fields, activity type vocabulary |
| 2: Signing | Ed25519 over 248 canonical bytes, nonce derivation, replay protection |
| 3: Registry | Signer identity, revocation by additive attestation, standing as graph |
| 4: Notarization | Solana Attestation Service binding, non-enumerable account layout |
| 5: Presentation | Verification endpoint, subject-authorized payload disclosure, refused operations |

All five layers together define the full conformance profile. An implementation may satisfy Layers 1 and 2 without notarizing (see the Postgres binding); that is a partial conformance and is honestly labeled as such.

## Neighboring specifications

Several existing specifications address signed factual claims: Verifiable Credentials (W3C), Verifiable Presentations (W3C), Open Badges 3.0 (IMS Global), C2PA (Coalition for Content Provenance and Authenticity), Sigstore (Linux Foundation, primarily for software artifacts). Each solves a real problem inside its target domain. This specification takes a narrower cut: it defines what a signed fact looks like as bytes, what provenance those bytes carry, and how a hash of those bytes is committed to a public substrate that verifiers can read without asking the platform of origin. What the substrate publishes is a payload-excluded, issuer-free envelope and a public hash that is not a directory.

## Status

Draft v0.2. Review happens via GitHub PRs and issues per [CONTRIBUTING.md](./CONTRIBUTING.md). A signature-collection surface for the review was considered and paused; a future revision may reintroduce it once independent implementations exist to sign against.

Prior version (v0.1-final) exists in git history at commit `608f25d` and earlier. Signatures over v0.1-final canonical bytes do not verify as v0.2; the byte layout changed at position 178 (`created_at` renamed to `signer_asserted_at`) and `spec_version` advanced from 2 to 3.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

Apache 2.0. See [LICENSE](./LICENSE).
