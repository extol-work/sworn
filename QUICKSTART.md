# SWORN in five minutes

Sign, verify, and reason about your first SWORN attestation before the
rest of your team gets back from lunch.

This is for implementers. If you want to read the spec first, start with
[README.md](./README.md) or [SPEC.md](./SPEC.md). If you want to attest
to the spec itself as part of the RFC review, see [RFC.md](./RFC.md).

## What you will do

Five commands. At the end you will have:

- An Ed25519 keypair that is your SWORN signer identity.
- A signed SWORN v0.1-final attestation, notarized in a local Postgres
  instance and independently verifiable off-chain against the spec.
- A working local implementation you can point third-party tools at, or
  extend, or read as a reference for building your own.

Total time: three to five minutes on a warm machine, ten minutes if
Docker has to pull images from scratch.

## Prerequisites

- **Docker** with `compose` (Docker Desktop, Colima, OrbStack, etc.).
- **Rust** 1.88 or newer if you want to build the CLI from source.
  If you'd rather skip Rust, the same operations work with `curl`
  against the HTTP API described in [openapi.yaml](https://github.com/extol-work/sworn-postgres/blob/main/openapi.yaml).

That's it. No accounts, no API keys, no blockchain wallets, no cloud
services.

## Five commands

```bash
# 1. Get the reference implementation
git clone https://github.com/extol-work/sworn-postgres.git && cd sworn-postgres

# 2. Bring up Postgres + the sworn-api server
docker compose up -d

# 3. Install the CLI (from source; ~2 min on cold cache)
cargo install --path cli

# 4. Generate a keypair and write a payload
sworn keygen > my.key
echo '{"kind":"endorsement","note":"my first SWORN attestation"}' > hello.json

# 5. Sign, submit, verify
sworn attest \
  --key my.key \
  --subject "sha256:$(shasum -a 256 hello.json | cut -d' ' -f1)" \
  --activity-type "sworn.dev/v1/endorsement" \
  --payload hello.json \
  --out attestation.json
sworn verify attestation.json
```

Expected on the last line:

```
valid:  true
reason: signature_verified
```

If that prints, you have a working SWORN implementation locally and a
signed attestation in your hands. If it does not, see
[Troubleshooting](#troubleshooting) below.

## What just happened

**Step 2** started two containers: Postgres 16 as the notarization
substrate and `sworn-api` as the HTTP surface that implements the SWORN
Layer 5 endpoints (SPEC §6). Migrations ran automatically; the
attestations, disclosure_tokens, and known_activity_types tables came up
empty.

**Step 3** built and installed the `sworn` CLI. The CLI wraps the HTTP
API. Every operation is also expressible via `curl` if you prefer the
wire directly.

**Step 4** generated a fresh Ed25519 keypair using your OS's
cryptographic random source (SPEC §3.2). That keypair is your persistent
SWORN signer identity. The public key is what appears in the `signer`
field of every attestation you sign; the private key never leaves your
machine.

**Step 5** did five things under the hood:

1. Loaded your keypair.
2. Computed `SHA-256` of the payload's RFC 8785 canonicalized form.
   That hash becomes the `data_hash` field of the attestation (SPEC §2.4).
3. Constructed the 248-byte canonical byte sequence per SPEC §3.1,
   including the `spec_version` marker, subject, hashes, provenance
   fields (defaults for self-reported), and a fresh random nonce.
4. Signed those 248 bytes with your Ed25519 private key using PureEdDSA
   (SPEC §3.2, deliberately NOT Ed25519ph).
5. Submitted the signed attestation to the local `sworn-api`, which
   re-verified the signature server-side and stored the row.

The `sworn verify attestation.json` step then re-verified independently
against the local file, without contacting the server. This is the
"durable, portable, independently verifiable" property in action: the
attestation is self-contained and re-checkable anywhere the Ed25519
signature scheme is available.

## Where to go next

**Sign an attestation about someone or something else.** Replace the
subject in step 5 with any 32-byte identifier that makes sense for what
you are attesting about. Any content hash (`sha256:...`), any Ed25519
public key of another SWORN signer (base64), or any 32-byte identifier
defined by an activity type's schema. See SPEC §2.6 for the subject
field's semantics.

**Attest with real provenance fields.** The default attestation above
is `source_type = self_reported`. To sign as an ORCID-sourced authorship
attestation, pass `--source-type orcid`, `--source-hash <sha256-of-orcid-id>`,
`--witnessing-depth computed_match`, `--confidence 9500`. See SPEC §2.5
and §9.2 for the field semantics and registered enum values.

**Try the two-call disclosure flow.** SWORN separates verification
(anyone can verify a signature and see metadata) from disclosure (the
payload requires signer-authorized single-use token). See SPEC §6.2 and
[sworn-postgres/scripts/quickstart.sh](https://github.com/extol-work/sworn-postgres/blob/main/scripts/quickstart.sh)
for a full 15-step exercise of the API surface.

**Cross-check your work against reference implementations.** Two
independent SWORN implementations ship in the spec repository at
[fixtures/runners/](./fixtures/runners/): one in Rust, one in Node.js,
sharing no code but producing byte-identical canonical bytes and
signatures against the same 5 golden vectors. If you build a third
implementation and it agrees with both, the spec text is doing its job.
If it disagrees, the divergence names either a spec bug or an
implementation bug and is legible either way.

**Verify against the HTTP conformance suite.** Once your implementation
exposes a SWORN HTTP surface, run
[fixtures/tests/rust/](./fixtures/tests/) against it to check tamper
detection, refused endpoints, provenance validation, and enum
fail-closed behavior. This is exactly the suite that caught a real
sourceless-zero-hash bug in the reference implementation's `verify()`
code the day it was written.

**Read the spec.** [SPEC.md](./SPEC.md) is the normative source. Every
opinion baked into the CLI or the reference implementation is a
translation of some section of SPEC.md. Reading them together is faster
than either one alone.

**Attest to the spec itself.** See [RFC.md §Attesting to the spec](./RFC.md#attesting-to-the-spec).
The RFC review process dogfoods SWORN at the deepest layer: reviewer
signatures are themselves SWORN attestations whose subject is a commit
hash of this repository.

## Troubleshooting

**Docker build fails on `cargo build`.** Rust dependency versions may
have moved past the Dockerfile's pinned Rust. Pull latest from
[extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres);
we bump Dockerfile Rust versions as needed. If you're still on the
wrong side after `git pull`, run `docker compose build --no-cache api`
to force a full rebuild.

**`cargo install --path cli` fails on `edition2024`.** Your local Rust
toolchain is older than 1.85. Update Rust via `rustup update stable`.

**`sworn verify` reports `signature_invalid`.** The most common cause
in Node-based re-implementations is that the crypto library defaults to
Ed25519ph (prehashed) rather than PureEdDSA. SPEC §3.2 requires
PureEdDSA specifically. See
[fixtures/runners/node/README.md](./fixtures/runners/node/README.md) for
the Node-specific fix (`crypto.sign(null, msg, key)`, not
`crypto.createSign('ed25519')`).

**`sworn verify` reports `payload_hash_mismatch`.** Your local
`attestation.json` has been modified since it was signed, or your JSON
canonicalization differs from RFC 8785. If you're re-implementing,
verify your canonicalization against the golden vectors at
[fixtures/attestations/v0.1-final/](./fixtures/attestations/v0.1-final/).

**HTTP 400 `refused` on a listing request.** Deliberate. See SPEC §6.4:
list-by-signer, list-by-subject, and bulk enumeration are refused
operations by design. If you need to enumerate for a specific
application resource identifier you already possess, see SPEC §6.4's
application-scoped exceptions carve-out.

**Something else.** Open an issue on
[extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres/issues)
if it's the reference implementation, or on
[extol-work/sworn](https://github.com/extol-work/sworn/issues) if it's
the spec text.
