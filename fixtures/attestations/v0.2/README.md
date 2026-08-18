# v0.2 reference vectors

**Status:** Informative reference material, not normative conformance suite.

This file is mirrored from the notary CLI reference implementation at
[github.com/extol-work/notary/fixtures/v0.2/vectors.json](https://github.com/extol-work/notary/blob/main/fixtures/v0.2/vectors.json).
It captures the byte-exact output of the reference implementation for six
representative attestations at `spec_version = 3` (v0.2).

## What this is

Six deterministic vectors, each specifying:

- **Input fields** (`input_fields`): the raw values that go into the canonical
  byte sequence per SPEC §3.1.
- **Signer secret seed** (`signer_secret_seed_hex`): the 32-byte Ed25519 seed
  used to sign this vector's canonical bytes.
- **Expected canonical bytes** (`expected_canonical_bytes_hex`): the 248-byte
  sequence produced by SPEC §3.1's layout rules.
- **Expected signature** (`expected_signature_hex`): the 64-byte Ed25519
  signature per SPEC §3.2 (PureEdDSA, no pre-hashing).

The six vectors cover meaningful edge cases: ORCID-sourced authorship
(baseline with all fields populated), self-reported sourceless (source_hash
and witness_for both zero), peer-witnessed physical observation, a
witness_for endorsement, an OAuth-authenticated attestation with default
retention, and an additive revocation per SPEC §4.3.

## What this is NOT

**Not the conformance suite.** v0.2.1 retreated from the framing that
positioned SPEC.md as a submitted protocol with verified interoperability
tests. See SPEC.md §10 (Implementation checklists) and the §1 preamble.
An implementation whose output byte-matches these vectors is exercising
the reference implementation's exact behavior; it is not thereby certified
as conforming to a protocol we have not yet submitted.

**Not authoritative in the case of drift.** These vectors are regenerated
from the notary CLI's baked inputs (`notary/src/vectors.rs`). If SPEC.md
changes the canonical byte layout at the same `spec_version`, this file
would need regeneration. Any implementation that reads these vectors as
its ground truth is reading a snapshot of the reference implementation's
output at the time the mirror was last refreshed.

## Regenerating

From a checkout of `github.com/extol-work/notary`:

```
cargo run -- vectors emit --out fixtures/v0.2/vectors.json
```

Then copy the file to `fixtures/attestations/v0.2/vectors.json` in the spec
repo. The two files should be byte-identical when the mirror is up to date.
