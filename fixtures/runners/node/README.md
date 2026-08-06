# SWORN v0.1-final vector-check runner (Node.js)

Independent Node.js implementation of the SWORN canonical byte sequence and
Ed25519 signing path. Reads the golden vectors in
`fixtures/attestations/v0.1-final/vectors.json` and verifies each byte-for-byte
against the spec.

**Deliberately self-contained.** No dependency on the [sworn-postgres][sp]
reference implementation. No dependency on any Extol production code. No npm
dependencies at all. Anyone reviewing the SWORN spec should be able to read
`src/canonical.mjs` alongside SPEC §3.1 and §3.2 and confirm the two agree,
without traversing a library graph.

[sp]: https://github.com/extol-work/sworn-postgres

Companion to [`../rust/`](../rust/), the same runner in Rust. Running both
against the same vectors and getting matching results confirms that SWORN's
byte-level serialization is unambiguous enough for independent
implementations in independent language ecosystems to agree.

## Usage

```bash
# from this directory
node src/check.mjs ../../attestations/v0.1-final/vectors.json

# or via the package.json script
npm run check -- ../../attestations/v0.1-final/vectors.json
```

The path argument is required. There is no default. A runner that silently
checks the wrong vectors is worse than one that refuses to run.

Requires Node.js 18.0 or later. No install step needed; `.mjs` files run
directly under Node with no transpilation. The `package.json` exists only
to declare `type: "module"` and the `check` script alias — no dependencies.

## Expected output

```
sworn-vector-check: v0.1-final (5 vectors)
  fixtures: ../../attestations/v0.1-final/vectors.json

  ✓ orcid_authorship_happy_path      canonical ✓  signature ✓  verify ✓
  ✓ self_reported_sourceless         canonical ✓  signature ✓  verify ✓
  ✓ peer_witnessed_physical          canonical ✓  signature ✓  verify ✓
  ✓ backfilled_migration             canonical ✓  signature ✓  verify ✓
  ✓ all_zero_provenance_edge_case    canonical ✓  signature ✓  verify ✓

5/5 passed.
```

Exits `0` if every vector passes all three checks. Exits `1` with per-vector
diagnostics otherwise. Exits `2` on argv, file, or vector-file schema errors.

## Per-vector diagnostics

Each vector is reported against three independent checks:

| Check       | What it proves                                                              | If it fails you have                                                        |
|-------------|-----------------------------------------------------------------------------|-----------------------------------------------------------------------------|
| `canonical` | Reconstructed 248-byte canonical bytes match `expected_canonical_bytes_hex` | A serialization bug: field order, endianness, or a missing/extra field       |
| `signature` | Signing the canonical bytes with `signer_secret_seed_hex` reproduces `expected_signature_hex` | A signing-path bug: PureEdDSA vs Ed25519ph, seed vs expanded key, wrong nonce derivation |
| `verify`    | `expected_signature_hex` verifies against the reconstructed canonical bytes and the signer pubkey | A verifier-side pubkey handling bug, or a transcription error in the vector itself |

Reporting the three separately is deliberate: hash-then-verify (SPEC §3.1.2)
is the two-sided check that distinguishes serialization drift from signing
drift from verifier configuration errors. A runner that collapses the three
into one "did it verify" boolean buries the interesting failure modes.

## Ed25519 note (Node.js)

Node's Ed25519 support is exposed through two very different APIs, one of
which is wrong for SWORN.

**Correct (PureEdDSA):**

```js
import { sign, verify, createPrivateKey } from "node:crypto";
const sig = sign(null, messageBytes, keyObject);
```

The literal `null` for the algorithm argument means "no hash pre-application,
sign the message directly." That is PureEdDSA per RFC 8032 §5.1, which is
what SPEC §3.2 requires.

**Wrong (Ed25519ph):**

```js
import { createSign } from "node:crypto";
const sig = createSign("ed25519").update(msg).sign(key);  // DO NOT USE
```

`createSign('ed25519')` prehashes the message before signing, producing
Ed25519ph signatures. SPEC §3.2 explicitly forbids Ed25519ph. A runner
built on `createSign` will fail every `signature` check against these
fixtures even if its canonical bytes are correct — this is the most
common Ed25519 confusion in Node and worth checking first if all your
signatures are wrong by a full 64 bytes.

The same distinction applies to `crypto.verify` (correct) vs
`crypto.createVerify` (wrong).

## Raw seed vs KMS-backed keys

The vectors ship the signer's raw 32-byte seed for reproducibility. Real
implementations often hold the private key behind a KMS or hardware boundary
and never see the raw seed. That's fine — the vectors exist to prove the
canonical byte layout and Ed25519 signing path are correct, not to demand a
specific key custody model. Any signing path that produces a byte-identical
signature over the same canonical bytes passes the T-5 test regardless of
whether the private key lived in memory, in an HSM, or in a KMS-backed
signing service.

## Reading the source

Two files, both dependency-free:

- `src/canonical.mjs` — canonical byte serializer for v0.1-final (248 bytes)
  and the deprecated v0.1-preview (208 bytes, reader-side only). Also
  provides a version-aware `serializeCanonicalBytes(input, specVersion)`
  dispatch that fails closed on unknown spec_versions with a distinct
  error type per IMPLEMENTATION_NOTES.md.
- `src/check.mjs` — the runner: reads the vectors, drives the serializer
  and Ed25519 sign+verify path, reports per-vector diagnostics.

Both are pure JavaScript with JSDoc type annotations for clarity. Nothing
here requires TypeScript, a build step, or a package manager beyond Node
itself.
