# SWORN reference runners

Independent implementations of the SWORN canonical byte sequence (SPEC §3.1)
and Ed25519 signing path (SPEC §3.2), used to verify golden vectors in
`../attestations/v0.1-final/vectors.json` byte-for-byte.

Two runners ship alongside the spec so any implementer can cross-check
against two independently-authored implementations rather than one:

- [`rust/`](rust/): self-contained Rust binary.
- [`node/`](node/): self-contained Node.js script.

Neither runner depends on the [sworn-postgres][sp] reference implementation
or on any Extol production code. Both were written to the spec text.

[sp]: https://github.com/extol-work/sworn-postgres

## Why two

Golden vectors are cheap. Runners against them are cheap. Having a second
runner in a different language costs almost nothing and catches spec
ambiguities the first runner might have quietly resolved by convention.

If your third implementation, in whatever language you like, produces
byte-identical canonical bytes and byte-identical signatures against these
vectors when the Rust and Node runners also pass, the spec text is doing
its job. If any of the three diverge, the divergence names a spec bug or
an implementation bug and either one is a fixable, legible failure.

## What each runner exercises

Both runners implement two SWORN conformance tests from SPEC §10.2:

- **T-1** (Verifier): the signature in `expected_signature_hex` verifies
  against the reconstructed canonical bytes and the signer pubkey.
- **T-5** (Signer): signing the reconstructed canonical bytes with the seed
  from `signer_secret_seed_hex` reproduces `expected_signature_hex`
  byte-for-byte.

T-2 through T-9 exercise the HTTP surface of a running implementation and
live in [`../tests/`](../tests/).

## Adding a third runner

Same shape. Read the spec, implement the canonical byte layout from §3.1,
sign with Ed25519 PureEdDSA per §3.2, run the same vectors, report per-vector
diagnostics with the three checks (`canonical`, `signature`, `verify`)
reported separately. Take the vector-file path as a required argv, no
defaults, no silent success against the wrong vectors.

If your first attempt produces a `signature` mismatch on every vector, the
most likely cause is your Ed25519 library defaulting to Ed25519ph (prehashed)
rather than PureEdDSA. See the "Ed25519 note" section in each runner's
README for the fix.
