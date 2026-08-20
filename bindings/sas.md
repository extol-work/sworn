# Binding: Solana Attestation Service (SAS)

**Status:** Normative. This document defines the concrete requirements a Layer 4 (Notarizer) conforming implementation MUST satisfy when using Solana Attestation Service as the notary substrate. Requirements in SPEC.md §5 govern what any notary MUST publish and MUST NOT permit; this document specifies how to satisfy them on SAS specifically.

**Prerequisites.** Reader familiarity with SPEC.md §3 (canonical byte sequence), §5 (notarization requirements), and the Solana Attestation Service program interface at `22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG` (mainnet-beta program ID as of the drafting of this document).

**Notation.** SAS terminology is used per the SAS program documentation: *credential* names the authority under which attestations are created; *schema* defines the shape of attestation data; *attestation account* is the on-chain record.

---

## §1 Why Solana Attestation Service

The engineering rationale for binding the specification's Layer 4 to Solana specifically is documented in PRIMER.md. In brief:

- SAS is a first-class Solana program providing credential, schema, and attestation-account primitives without requiring per-adopter contract deployment.
- Solana signs with Ed25519 natively, matching the specification's Layer 2 requirement without algorithm bridging.
- Solana's Program Derived Address model allows caller-chosen opaque seeds, which is what makes SPEC.md §5.1's non-walkability discipline achievable.
- Per-transaction fees (approximately $0.0006) and sub-5-second finality match the interaction pattern conforming attestations are used in. Per-attestation account-rent cost is separate and is discussed in §11.

A future substrate satisfying the same properties could be added as an additional binding through a subsequent version of SPEC.md. None currently qualify. Adopters who wish to deploy this specification on another substrate should read this document as the reference for what the corresponding substrate binding needs to establish.

## §2 Credential setup

**Requirement.** Each conforming deployment MUST operate under exactly one SAS credential per environment (mainnet-beta, devnet, testnet). The credential authority is the party responsible for ensuring published attestations conform to SPEC.md §5.

The credential is created once per environment via SAS's `CreateCredential` instruction. The credential authority's public key is the fee payer for all notary transactions in that environment.

**Authorized signers list.** The credential's authorized-signers list MUST contain only the deployment's notary signer public keys. Adding an authorized signer is a governance action; the deployment operator MUST document its process for adding, rotating, and removing authorized signers. The specification itself does not specify this process; deployments MAY use single-signer, multisig, KMS-managed, or hardware-security-module patterns as their threat model requires.

**Credential name.** conforming deployments SHOULD use a credential name that clearly identifies the deployment (e.g., `extol-mainnet-v1`, `example-org-prod`) rather than a generic name.

## §3 Schema setup

**Requirement.** Each conforming deployment MUST use exactly one SAS schema per environment. Multiple schemas within one deployment is not permitted in v0.2; a future version may specify a schema versioning discipline.

The schema is created once per environment via SAS's `CreateSchema` instruction, under the credential from §2.

### §3.1 Schema data layout

The schema's data section carries three fields whose semantic payload is 42 bytes and whose on-wire encoding is 46 bytes:

| Field | Semantic offset | Semantic length | Encoding |
|---|---|---|---|
| `spec_version` | 0 | 2 bytes | u16 little-endian, always equals SPEC.md §3.1.1's registered spec_version value at attestation time (currently 3 for v0.2). |
| `attestation_hash` | 2 | 32 bytes | `SHA-256(canonical_bytes)` per SPEC.md §3.1. |
| `signer_asserted_at` | 34 | 8 bytes | int64 little-endian Unix seconds, copied verbatim from the attestation's `signer_asserted_at` field. |

Semantic total: 42 bytes. On-wire total: 46 bytes (see below).

**Schema layout type codes.** The SAS `CreateSchema` instruction
requires a `layout: Vec<u8>` argument specifying the SchemaDataTypes
enum value for each field. For the conforming layout:

| Field | SchemaDataTypes value | Byte |
|---|---|---|
| `spec_version` | U16 | 1 |
| `attestation_hash` | VecU8 | 13 |
| `signer_asserted_at` | I64 | 8 |

Layout: `[1, 13, 8]`. Field names in order:
`["spec_version", "attestation_hash", "signer_asserted_at"]`.

**On-wire size.** SAS's SchemaDataTypes enum has no fixed-length byte
array type. The 32-byte `attestation_hash` is encoded as `VecU8`,
which adds a 4-byte little-endian length prefix. On-wire size is
therefore 46 bytes, not 42:

    offset  0.. 2   spec_version         (2 bytes, u16 LE)
    offset  2.. 6   attestation_hash len (4 bytes, u32 LE, value = 32)
    offset  6..38   attestation_hash    (32 bytes)
    offset 38..46   signer_asserted_at  (8 bytes, i64 LE)

The 42-byte figure in earlier drafts referred to the semantic payload
(2 + 32 + 8); the on-wire encoding is 46 bytes including the length
prefix. Reference implementations MUST encode 46 bytes.

**Nothing else appears in the schema data section.** The schema is deliberately minimal; no signer, subject, activity_type, source_hash, witness_for, or provenance fields are stored on-chain. Their presence in the account data would defeat SPEC.md §5.1's non-walkability requirement, since SAS accounts are queryable by content via `memcmp` filters.

**Rationale.** The 46-byte on-wire payload (42 bytes semantic content) is sufficient for a verifier who possesses the attestation off-chain to confirm anchoring: recompute `SHA-256(canonical_bytes)`, look up the corresponding PDA (see §4), read the account, compare the on-chain hash and version, and read the Solana block time as the notarization timestamp. It is insufficient for anyone scanning the SAS program to reconstruct who signed what.

**Reference implementation.** The layout type codes, on-wire encoding, and `CreateSchema` invocation are implemented in [`extol-work/notary/src/sas.rs`](https://github.com/extol-work/notary/blob/main/src/sas.rs). See [REFERENCE_IMPLEMENTATIONS.md](../REFERENCE_IMPLEMENTATIONS.md) for other implementations of this binding.

### §3.2 Schema description field

The schema's description field (if the SAS deployment supports one) SHOULD contain the string `v0.2 notary anchor: spec_version || attestation_hash || signer_asserted_at`. This is informative metadata; it does not affect verification.

## §4 PDA seed derivation

**Requirement.** The SAS attestation account PDA for a conforming attestation MUST be derived as:

```
PDA seeds = [
    b"attestation",
    credential_pubkey,
    schema_pubkey,
    nonce_pubkey,
]
```

where `nonce_pubkey` is `SHA-256(canonical_bytes)` interpreted as a 32-byte pubkey (may or may not be on the Ed25519 curve; SAS does not require it to be).

Using `SHA-256(canonical_bytes)` as the nonce means:

- The PDA is deterministic: any party holding the canonical bytes can compute the PDA without any state lookup.
- The PDA is opaque with respect to signer, subject, or any other field: a scan of the SAS program's accounts yields a set of hashes with no signal about their contents.
- The PDA is idempotent: attempting to publish the same attestation twice produces the same PDA, so SAS's account-already-exists check prevents duplicates without any additional state on the the specification side.

**Non-conforming derivations.** The following seed patterns violate SPEC.md §5.1 non-walkability and MUST NOT be used:

- Including the signer's pubkey in seeds
- Including the subject's pubkey in seeds
- Including activity_hash, source_hash, or witness_for in seeds
- Including any hash whose preimage is one of the above fields, plus a limited-entropy discriminator that would let a scanner brute-force reverse the hash

**The old Extol derivation was non-conforming.** Prior to v0.2, the Extol mainnet deployment used a nonce derived from `(activity_hash, subject, attestation_type, witness_for)`. This exposed signer and subject to bulk substrate scans and does not conform to v0.2's §5.1. See §7 (Migration) below.

## §5 Attestation creation

**Requirement.** conforming attestations are notarized via the SAS `CreateAttestation` instruction, invoked by a notary signer authorized under the credential from §2.

The instruction MUST be invoked with:

- The credential and schema from §2 and §3.
- The nonce from §4.
- Attestation data of exactly 46 bytes on wire matching the schema layout in §3.1 (42 bytes semantic payload plus the 4-byte length prefix for the `VecU8` attestation_hash field).
- The notary signer as the transaction fee payer.

The conforming attestation's `signer` is NOT the SAS credential authority. The signing layer and SAS have different signing models; the attestation signature (Ed25519 over 248 canonical bytes per SPEC.md §3) is produced by the attestation's signer and lives off-chain, while the SAS transaction signature is produced by the notary signer authorized under the credential.

### §5.1 Required instruction parameters

The `CreateAttestation` instruction accepts an `expiry` field (i64 Unix
seconds, per SAS's instruction data layout). Conforming deployments
MUST pass `expiry = 0` on every `CreateAttestation` call, which SAS
interprets as "never expires." Any nonzero expiry value would authorize
SAS to accept a later `CloseAttestation` call against the account
purely on the basis of elapsed time, which would violate SPEC §5.4's
durability commitment.

This is a substrate-level enforcement of one class of the operator
commitment; authority-initiated Close remains reachable and is
addressed by SPEC §5.4's operator commitment rule.

**Sign locally, notarize via a service.** Because the notary signer is not the attestation signer, the common deployment pattern is:

1. The attestation signer produces the canonical bytes and Ed25519 signature off-chain (typically in a client or KMS-mediated flow).
2. The signed attestation is transmitted to a notary service operated by the credential authority.
3. The notary service computes `SHA-256(canonical_bytes)`, verifies the attestation signature to ensure the attestation is well-formed, and invokes `CreateAttestation` with the notary signer as fee payer.
4. The notary hash is now anchored.

This pattern lets clients hold their own signing keys without also holding SOL for transaction fees.

## §6 Forbidden SAS instructions

A conforming conforming deployment MUST NOT invoke the following SAS instructions on conforming attestations:

- **`CreateTokenizedAttestation`** and any tokenization variant. conforming attestations are not fungible or non-fungible tokens; presenting them as tokens contradicts the fact-signing model of the specification and violates SPEC.md §5.4 (durability) if the tokenized form permits transfer that changes ownership independent of the underlying attestation's signer.
- **`CloseAttestation`.** Once anchored, a conforming attestation MUST remain published for as long as the SAS substrate maintains any record per SPEC.md §5.3 and §5.4. Invoking `CloseAttestation` would break the durability property.
- **`ChangeSchema`** or any instruction that mutates a previously-anchored attestation's data. SPEC.md §5.4 forbids notary hash mutation.

Deployments MAY use SAS's other administrative instructions (`ChangeAuthorizedSigners` for authorized-signer rotation, credential and schema management) as needed, provided they do not violate SPEC.md's durability or non-enumerability rules.

**Implementer note.** SAS accepts these instructions and does not distinguish conforming attestations from other SAS uses at the program layer. Enforcement is deployment discipline: the notary service code path MUST NOT construct or submit these instructions against conforming attestation accounts.

## §7 Merkle batching

Some deployments batch attestation hashes into a Merkle tree and anchor the root as a single SAS attestation. This is permitted by SPEC.md §5.2 with the following requirements.

**Merkle root anchoring.** The Merkle root is anchored as a distinct SAS attestation whose schema data section holds:

| Field | Offset | Length | Encoding |
|---|---|---|---|
| `spec_version` | 0 | 2 bytes | u16 little-endian, same as §3.1. |
| `merkle_root` | 2 | 32 bytes | Root of the Merkle tree over member `SHA-256(canonical_bytes)` values. |
| `batch_asserted_at` | 34 | 8 bytes | int64 Unix seconds, notary's assertion of the batch time. |

Same 42-byte semantic payload (46 bytes on wire, per §3.1's `VecU8` encoding rule). Merkle root anchors are distinguishable from individual attestation anchors only by application context; the on-chain byte structure is identical.

**Inclusion proofs.** For each member attestation in a batch, the notary MUST provide, on request from any verifier who possesses the canonical bytes:

- The 32-byte member hash (`SHA-256(canonical_bytes)`).
- The Merkle proof (a sequence of sibling hashes) that resolves to the batched root.
- The PDA of the anchored Merkle root.

**Verification procedure.** A verifier presented with a member attestation and inclusion proof:

1. Computes `SHA-256(canonical_bytes)` of the member attestation.
2. Walks the Merkle proof to recompute the root.
3. Reads the anchored Merkle root PDA from SAS and confirms the recomputed root matches the on-chain 32 bytes at semantic offset 2 (on-wire offset 6, after the `VecU8` length prefix).
4. Reads the SAS transaction's block time as the notarization timestamp for the entire batch.

**Merkle tree construction.** For v0.2, this specification does not normatively define the Merkle construction algorithm (binary vs unbalanced, hash prefixing for depth safety, node encoding). Implementations MUST document their construction such that a third-party verifier receiving an inclusion proof can recompute the root without out-of-band information.

A normative Merkle construction is targeted for v0.3.

## §8 Read and verification procedure

A verifier who possesses a conforming attestation (canonical bytes plus signature) and wants to confirm it is anchored on SAS:

1. Compute `attestation_hash = SHA-256(canonical_bytes)` per SPEC.md §3.1.
2. Compute the SAS attestation PDA per §4 of this document using the deployment's credential and schema pubkeys plus `attestation_hash` as the nonce.
3. Fetch the account at the computed PDA via any Solana RPC.
4. If the account exists, read its schema data section (46 bytes on wire per §3.1):
   - Confirm the `spec_version` at on-wire offset 0 matches the attestation's spec_version.
   - Skip the 4-byte `VecU8` length prefix at on-wire offset 2 (value = 32).
   - Confirm the `attestation_hash` at on-wire offset 6 matches the computed value from step 1 (byte-for-byte).
   - Read the `signer_asserted_at` at on-wire offset 38; the verifier MAY compare it to the block time of the transaction that created the account to detect signer-clock inaccuracy.
5. If the account does not exist, check whether the attestation is anchored via a batched Merkle root (§7). This requires the verifier to have obtained an inclusion proof out of band.
6. Read the Solana block time for the account-creating transaction as the authoritative notarization timestamp per SPEC.md §2.7.

If steps 1 through 4 succeed (or 5 succeeds for batched attestations) and the block time is within the verifier's acceptance policy, the attestation is durably anchored.

**No signer or subject appear in this procedure.** The verifier must possess the attestation's canonical bytes to compute the PDA. This is the mechanism by which SPEC.md §5.1's non-walkability discipline is enforced: without the canonical bytes, the PDA is not derivable, and a scan of the SAS program yields no useful information.

## §9 Migration from pre-v0.2 deployments

Deployments that anchored attestations under pre-v0.2 PDA derivations have those attestations in a non-conforming state per this binding. Two migration options are available.

**Option A: Grandfather.** Pre-v0.2 attestations remain readable under the old PDA derivation using the deployment's pre-v0.2 code path. The deployment retains that code path for legacy reads while adopting the v0.2 derivation for all new attestations. Pre-v0.2 attestations become unreachable via v0.2 read tools, but their signatures remain cryptographically valid and can still be verified off-chain via SPEC.md §3.1's verification procedure.

**Option B: Re-anchor.** Pre-v0.2 attestations are re-anchored under the v0.2 PDA derivation. Original anchor timestamps are lost (the re-anchor establishes a new Solana block time). This option is only appropriate for deployments where original timestamps do not matter for downstream verification.

**Extol's chosen option.** Extol's mainnet deployment adopts Option A. Pre-v0.2 mainnet attestations remain anchored under the pre-v0.2 derivation; new attestations use v0.2 derivation. The migration engineering ticket is EXT-247 in Extol's internal tracker.

Third-party deployments MAY choose either option. Both are conforming to v0.2 for new attestations; the choice affects legacy attestations only.

## §10 Program IDs and account addresses

| Environment | SAS Program ID | Notes |
|---|---|---|
| Mainnet-beta | `22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG` | Production. |
| Devnet | `22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG` | Test environment. |

Each deployment publishes its credential and schema pubkeys in deployment-specific documentation. Verifiers who want to confirm anchoring against a specific deployment need the deployment's credential and schema pubkeys as inputs to §4's PDA derivation.

## §11 Cost economics

**Per-attestation costs.** Each individual `CreateAttestation` incurs:

- A rent-exempt SOL deposit for the SAS attestation account (~0.002 SOL
  for a 215-byte account, approximately $0.30 at recent SOL prices).
  This deposit is locked for the life of the account. Under SPEC §5.4's
  operator commitment (no authority-initiated Close), this locked
  deposit is effectively permanent.
- Transaction fees (~5000 lamports per transaction, approximately
  $0.0006 at recent SOL prices).

**Total per individual attestation:** approximately $0.30, dominated
by the rent deposit rather than the transaction fee. A deployment
anchoring N individual attestations locks approximately N × $0.30 in
SOL, permanently.

**Solana reduced-rent upgrade (in progress).** The Solana network has
begun a phased reduction of `lamports_per_byte` from 6,960 to 696 (a
90% reduction), rolling out across five feature gates on Agave 4.2
starting August 2026 with a sixth feature gate that permits reversion
if the rollout surfaces cluster health concerns. If the full reduction
lands, the per-attestation rent deposit drops from approximately $0.30
to approximately $0.03. The economic argument for Merkle batching
weakens by a factor of ten but does not disappear: at scale
(millions of attestations), $30K locked is still qualitatively
different from $300 locked. See [solana.com/upgrades/reduced-rent](https://solana.com/upgrades/reduced-rent)
for current activation status. Implementations SHOULD NOT rely on the
reduction until all five feature gates have activated on the deployment's
target cluster.

**Merkle batching is the production path for volume deployments.** A
deployment anchoring more than approximately 100 attestations per day
SHOULD use Merkle batching (§7) as the primary anchoring path. Under
Merkle batching, a single SAS attestation account (one $0.30 locked
deposit, or ~$0.03 post-reduction) anchors thousands of member hashes;
the amortized locked cost per member attestation drops to fractions of
a cent under either rent regime.

Individual anchoring remains appropriate for low-volume deployments
where the batching operational overhead (Merkle tree construction,
inclusion proof storage, off-chain proof serving) is not justified.

**Rate limits.** SAS itself does not rate-limit. Solana RPC providers
rate-limit at the transaction submission layer. Deployments SHOULD
queue notary transactions and use exponential backoff on RPC failures.

---

## Appendix A: Reference implementations

Reference implementations of this specification — portable references for the canonical bytes and SAS binding, and Extol's own operator deployment reference — are listed in [REFERENCE_IMPLEMENTATIONS.md](../REFERENCE_IMPLEMENTATIONS.md) at the repository root.

Where any Active implementation and this specification disagree, the specification wins and the implementation gets updated to close the gap. Historical implementations may permanently diverge from the current spec — that is what "historical" means.
