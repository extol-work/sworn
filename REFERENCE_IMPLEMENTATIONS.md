# Reference implementations

**Non-normative.** This file is a living record of implementations of the Attestation Notary Specification. It is not part of the spec's conformance definitions in [SPEC.md](./SPEC.md) §10 and does not confer conformance status on the implementations listed here. Anyone can submit an implementation via a pull request against this file.

Implementations differ in the layers they cover, the language they use, and the substrate they anchor to. Adopters looking for a starting point should read the ["Choosing a reference"](#choosing-a-reference) section before picking one.

## Current

| Implementation | Language | Scope | Notes | Status |
|---|---|---|---|---|
| [extol-work/notary](https://github.com/extol-work/notary) | Rust | Layers 1+2+4+5 | Portable full-stack reference implementation. CLI: `attest`. Source of truth for the 248-byte canonical layout, Ed25519 PureEdDSA signing, RFC 8785 payload hashing, activity_hash NFC discipline, PDA seed derivation per [bindings/sas.md](./bindings/sas.md), and disclosure-token issue/redeem per SPEC §6.3. Ships with [golden test vectors](./fixtures/) that every conforming implementation must reproduce byte-for-byte. | Active |
| [@extol-work/notary](https://www.npmjs.com/package/@extol-work/notary) | TypeScript | Layers 1+2 | npm package. Canonical byte serialization and Ed25519 verification for browser and Node consumers. Ports the Rust reference byte-for-byte; validated against the same golden vectors on every publish. Package source is in a currently-private repo; the npm-published tarball is the adopter-facing artifact. Layers 4 and 5 subpaths (`@extol-work/notary/sas`, `@extol-work/notary/disclose`) are reserved for future releases; adopters needing Layer 4 today should use the Rust CLI or integrate `@solana/web3.js` directly against the SAS binding. | Active |
| Extol Cortex | Rust + TypeScript | Layers 1+2+4 | ¹ Operator deployment reference. Extol's production notary deployment (source not currently public). Implements the specification correctly but includes Extol-specific product concerns (identity derivation from platform IDs, treasury coordination, callback routing to the brij application layer). Useful as a conceptual reference for operators; **not** a clean-room portable reference. Currently anchoring under v0.1-final; migrating to v0.2 per EXT-247. New adopters should treat the notary CLI as the current-generation portable reference until the migration completes. | Active |

¹ **On the Cortex row.** Cortex is listed as an active conforming deployment for adopters who want to know that at least one operator runs this specification in production. Cortex source is not currently public, so it is not a code-level reference; it is a conceptual reference for the operational shape of a real deployment. Cortex also carries product concerns that are not part of this specification (community model, wallet coordination, cross-service callback signing, tier-based rate limiting), which would make it a poor clean-room reference even if it were public. The `extol-work/notary` CLI is the code-level portable reference for the specification itself.

## Historical (retired)

Implementations that predate the current spec version. Preserved for reference against attestations signed under earlier versions of the specification. Not maintained against v0.2 or later.

| Implementation | Version pinned | Scope | Notes |
|---|---|---|---|
| [extol-work/sworn-postgres@b13c74d](https://github.com/extol-work/sworn-postgres/tree/b13c74d) | v0.1-final (spec_version = 2) | Layers 1+2 (Postgres binding) | Working example of Layer 1+2 partial conformance under v0.1-final, with a Postgres-backed anchoring surface. Not maintained against v0.2. Verifiers of pre-v0.2 attestations that were anchored to sworn-postgres can pin this SHA to reproduce the older canonical byte layout (`spec_version = 2`; the field at what is now `signer_asserted_at` was named `created_at` in v0.1-final). See the deprecation banner on the linked repo for details. |

## Choosing a reference

If you are building an implementation from scratch and want to know which of the above to consult:

- **You need canonical bytes and Ed25519 for a language other than TypeScript or Rust.** Consult `extol-work/notary` (`src/canonical.rs`, `src/attestation.rs`) and validate byte-for-byte against `fixtures/v0.2/vectors.json`. The Rust code is short, self-contained, and has no substrate dependencies at the canonical-bytes layer.

- **You are consuming attestations from a browser or Node.js application.** Use `@extol-work/notary` from npm. Its verify path matches the Rust reference byte-for-byte on every published version.

- **You are implementing a SAS binding for your own operator deployment.** Consult `extol-work/notary` (`src/sas.rs`) for the reference PDA derivation and instruction encoding. The specification's [bindings/sas.md](./bindings/sas.md) is the normative source; the CLI code helps translate MUST clauses into working transactions.

- **You are implementing a non-SAS binding** (a different Layer 4 substrate). The Postgres binding at [bindings/postgres.md](./bindings/postgres.md) documents one non-SAS approach at partial conformance. The `sworn-postgres` historical implementation is a working example of that approach under v0.1-final; a fresh Layer 1+2 implementation against v0.2 would be a welcome addition to this list.

## Adding an implementation

Open a pull request against this file adding a row to either the Current or Historical table, with:

- Name and repository link
- Language
- Layers covered
- A pointer to your test suite or golden-vector conformance runner
- Your maintenance intent, using one of:
  - **Active** — you keep the implementation aligned with the current spec version and intend to update it when the spec advances.
  - **Best-effort** — you maintain the implementation as time allows; adopters should not assume immediate spec-alignment on version bumps.
  - **Reference-only** — you consider the implementation feature-complete at its current spec version and do not intend further updates; behaves as a Historical row from adopters' point of view even if the spec version currently matches.

Implementations that pass the golden vectors at `fixtures/v0.2/vectors.json` byte-for-byte are welcome regardless of language, substrate, or organizational affiliation. Passing conformance is a matter of matching bytes, not being endorsed by Extol.
