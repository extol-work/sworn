# Attestation Notary Specification, draft v0.2

**Status:** Draft. Under revision from v0.1-final following review; not yet accepting signatures against this text.

**Notation.** The words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY carry their RFC 2119 meaning throughout.

**Companion documents.** PRIMER.md explains the design intent and history. bindings/sas.md defines the required Solana binding. bindings/postgres.md documents a partial-conformance Postgres binding. This document is the only normative artifact.

---

## §1 Overview

### §1.1 What this specifies

This specification defines how a signer produces a signed statement of fact, how that statement's cryptographic identity is committed to a public ledger, and how a later verifier confirms both the signature and the ledger commitment without trusting the platform that produced the statement.

A conforming attestation has three properties:

- **Signed.** A specific signing key produced a signature over a specific canonical byte sequence.
- **Notarized.** The attestation's identifying hash is committed to a public, tamper-evident substrate at a substrate-native timestamp any verifier can read.
- **Independently verifiable.** Any party holding the attestation and the substrate's public state can confirm both properties without further permission or coordination.

That is the whole specification. Interpretations of the attestation, its social meaning, its relationship to reputation or reward, and any use to which it is put by a downstream application, are all outside the specification. See §1.4 for the non-goals this document explicitly does not undertake.

### §1.2 Terminology

**Attestation.** A signed statement by one party (the *signer*) about another party or artifact (the *subject*). The statement commits to a payload via that payload's hash, together with metadata describing the class of statement, when it was made, and what source the signer relied on.

**Signer.** The Ed25519 public key that produced an attestation's signature. Signer identity is exactly the public key. Any mapping between a signer and a real-world person or organization is outside this specification.

**Subject.** The entity being attested about. May be another conforming signer's public key, an arbitrary public key, or a 32-byte content hash. The interpretation is defined by the attestation's activity type schema.

**Payload.** The semantic content of the attestation. A JSON object whose shape is defined by the activity type. The payload is not signed as bytes; the payload's canonicalized hash is.

**Activity type.** A URI naming the class of claim being made. The URI names a schema document that defines the payload's structure.

**Provenance.** The signer's claim about the origin of the attestation, captured by four fields on the canonical byte sequence: `source_type`, `source_hash`, `confidence`, `witnessing_depth`, and `attestor_relationship`. Provenance is a claim, not a guarantee (§2.5.1).

**Notarization substrate.** The public, tamper-evident ledger where attestation identifying hashes are committed. In v0.2 this is Solana Attestation Service (SAS) as defined in bindings/sas.md. Other substrates may satisfy Layer 1 and Layer 2 only (see bindings/postgres.md); they do not offer Layer 4 conformance.

**Conforming implementation.** Software that produces, stores, notarizes, and verifies attestations per this specification. See §10 for the levels of conformance.

### §1.3 Layer model

This specification is organized in five layers. A conforming implementation MUST implement all five.

- **Layer 1, Testimony (§2).** The structure of an attestation record.
- **Layer 2, Signing (§3).** How an attestation is bound to a signer.
- **Layer 3, Registry (§4).** Signer identity semantics and revocation.
- **Layer 4, Notarization (§5).** How attestation hashes are committed to Solana Attestation Service.
- **Layer 5, Presentation (§6).** How third parties verify attestations.

A partial-conformance implementation MAY implement Layers 1 and 2 only, without publishing to a substrate. Such an implementation produces signed attestations that any Layer 4 party can later notarize; it does not itself provide the notarization property.

### §1.4 What this specification does not specify

This specification does not define any of the following. Any product built on this specification that offers these properties does so above the specification, in its own documentation and under its own terms.

- **Witnessing as a protocol operation.** The specification produces signed statements by one party. A `witness_for` field exists in the record (§2.6) as an optional pointer to another party, but the protocol does not run a two-party ceremony and does not enforce anything about the referenced party's participation. Applications that want multi-party witnessing build it above this specification as composition of independent attestations.

- **Non-transferability.** A signed byte sequence is copyable and portable by its nature. This specification does not define an owner field, a transfer instruction, or any mechanism that would make one attestation belong to one key more than another. The Solana binding forbids the tokenize and close instructions on the underlying SAS attestations (bindings/sas.md §4), which prevents the substrate itself from expressing transfers or deletions; it does not prevent an application from constructing derived assets that reference these attestations. Non-transferability at the product layer is application policy, not specification property.

- **A scoring, ranking, or aggregate quality function.** The graph of attestations is a public record. Interpretations of it (how to weigh peer-witnessed against computed-match provenance, how to decay older attestations, how to compose several corroborations) belong to readers. Two implementations reading the same graph may compute different derived signals for legitimate reasons.

- **Real-world identity verification, proof-of-personhood, KYC.** The signer is a public key. Any binding of that key to a real-world identity is application-defined.

- **Roles, affiliation, delegation, multi-signature.** Reserved for future versions (§4.5).

### §1.5 Notational conventions

Byte-level fields are little-endian unless explicitly noted. All hashes are SHA-256 (FIPS 180-4) unless otherwise specified. All signatures are Ed25519 (RFC 8032, PureEdDSA per §3.2) unless a registered alternative algorithm is used (§3.3).

Public keys are 32-byte Ed25519 encoded points. Signatures are 64 bytes. Byte concatenation is denoted `||`.

Integer positions in the registries at §9.2 through §9.4 are stable across versions: once assigned, a position does not change. String labels for registered values may be renamed; integer values may not be renumbered.

---

## §2 Layer 1: Testimony

### §2.1 Attestation record structure

An attestation record is a tuple with the following fields:

| Field | Type | Length | Description |
|---|---|---|---|
| `spec_version` | u16 little-endian | 2 bytes | Specification version this attestation is signed against. See §3.1. |
| `signer` | Ed25519 public key | 32 bytes | Produced the signature. See §3. |
| `subject` | pubkey or content hash | 32 bytes | Entity being attested about. See §2.6. |
| `activity_type` | URI (UTF-8) | variable | Names the class of claim. See §2.2. |
| `data_hash` | SHA-256 | 32 bytes | Hash of the canonical payload. See §2.3–§2.4. |
| `witness_for` | pubkey OR 32 zero bytes | 32 bytes | Optional pointer to another party's claim. See §2.6. |
| `source_hash` | SHA-256 or 32 zero bytes | 32 bytes | Hash of the canonical source identifier. See §2.4–§2.5. |
| `source_type` | u16 little-endian | 2 bytes | The kind of source. See §2.5, §9.2. |
| `confidence` | u16 little-endian | 2 bytes | Signer's confidence estimate, 0–10000 basis points. See §2.5. |
| `witnessing_depth` | u8 | 1 byte | Depth of the witnessing act. See §2.5, §9.3. |
| `attestor_relationship` | u8 | 1 byte | Signer's relationship to the subject. See §2.5, §9.4. |
| `signer_asserted_at` | int64 Unix seconds | 8 bytes | When the signer claims to have signed. See §2.7. |
| `retention_hint` | int64 | 8 bytes | Payload retention hint. See §2.7. |
| `nonce` | opaque 32 bytes | 32 bytes | Per-attestation uniqueness. See §3.4. |
| `signature` | Ed25519 signature | 64 bytes | Over canonical bytes. See §3.1. |

Serialization for storage and transport is implementation-defined. A conforming implementation MUST be able to reconstruct the canonical byte sequence (§3.1) from stored data.

Not signed over but required in a complete record: the `payload` (whose hash is `data_hash`) and any off-chain metadata annotations (see §2.8).

### §2.2 Activity type namespacing

An `activity_type` is a URI naming the class of claim being made. The URI MUST be:

- absolute (including scheme);
- resolvable in principle to a schema document describing the payload structure;
- stable, meaning implementations MUST NOT change the meaning of an existing URI. Schema evolution requires a new URI, typically with a version segment.

Examples of well-formed activity types:

- `https://schemas.example.org/statement-of-service/v1`
- `https://credit.niso.org/contributor-roles/writing-original-draft/`

Namespace prefixes MAY be URL-style. Reverse-DNS style (`org.example.foo`) is permitted as a URI scheme identifier where the receiving implementation registers such a scheme; conforming implementations without such a registration MUST reject reverse-DNS strings that lack a scheme.

**Extension.** Any party MAY define a new activity type by publishing a schema at the URI. There is no central registry in v0.2; §9.1 documents namespaces already in use so implementers can avoid collision. Willingness of a signer to use an activity type and willingness of a verifier to interpret it are the coordination mechanism.

**Established vocabularies.** Where an established vocabulary already exists for a domain (for example, the CRediT contributor role taxonomy for research contributions, defined by ANSI/NISO Z39.104-2022), implementations SHOULD adopt it rather than mint a parallel namespace.

### §2.3 Canonical JSON encoding for the semantic payload

The payload is a JSON object whose shape is defined by the activity type's schema. For the payload to be hashable in a deterministic, cross-implementation manner, it MUST be canonicalized before hashing per RFC 8785 (JSON Canonicalization Scheme):

1. Encode all strings as UTF-8 with Unicode NFC normalization.
2. Sort object keys lexicographically by their UTF-16 code units.
3. Serialize numbers per RFC 8785 §3.2.2.3.
4. Emit no insignificant whitespace.
5. Represent booleans as `true` / `false` and null as `null`.

Implementations MUST produce identical bytes for identical semantic payloads regardless of language, library, or platform.

### §2.4 activity_hash, data_hash, and source_hash primitives

Three hashes appear in the canonical byte sequence:

- **`activity_hash`** (32 bytes, derived not stored separately): `SHA-256(UTF-8 bytes of the activity_type URI, after Unicode NFC normalization)`. Implementations that store the URI derive this hash on demand.

- **`data_hash`** (32 bytes): `SHA-256(canonicalized JSON payload)` per §2.3.

- **`source_hash`** (32 bytes): `SHA-256(canonical source identifier)` per §9.2's per-source-type canonicalization rules.

**Sourceless attestations.** When `source_type ∈ {0, 1}` (`unknown` or `self_reported`), `source_hash` MUST be exactly 32 zero bytes. Signers MUST NOT populate `source_hash` for sourceless attestations. Verifiers MUST reject any attestation whose `source_type` is 0 or 1 but whose `source_hash` is nonzero.

### §2.5 Provenance fields

The four provenance fields (`source_type`, `source_hash`, `confidence`, `witnessing_depth`, `attestor_relationship`) commit the signer's claim about the origin and method of the attestation. Registered values are in §9.2 through §9.4.

**`source_type`** (u16) identifies the kind of source. Sixteen values are registered as of v0.1.1 (positions 0 through 15). Additions are additive and do not advance `spec_version`.

**`source_hash`** (32 bytes) identifies the specific source per §2.4.

**`confidence`** (u16) is the signer's estimate in basis points from 0 to 10000. A value of 10000 means "as confident as I can be given this source." A value of 0 means "I have no confidence in this claim." Confidence is a snapshot at signing time; it MUST NOT be revised in place. A signer who later learns their claim was weaker SHOULD issue a new attestation with corrected confidence, using the additive-attestation pattern of §4.3.

**`witnessing_depth`** (u8) captures the depth of the witnessing act itself, orthogonal to source type. Six values are registered.

**`attestor_relationship`** (u8) captures the signer's relationship to the subject at signing time. Seven values are registered. This is a snapshot; changes in relationship over time do not retroactively alter signed attestations.

#### §2.5.1 Signer's claim, not verifier's guarantee

Provenance fields express the signer's claim about the source and method. A verifier receiving an attestation with `source_type = orcid` and `confidence = 9500` MUST treat this as the signer's assertion, not the verifier's guarantee. Independent verification of high-confidence sources is a verifier-side responsibility, not a spec-level requirement.

Implementations MUST NOT reject attestations solely because their provenance is not independently verifiable. Confidence-based reader-side heuristics are implementation-defined.

#### §2.5.2 Dedup semantics

Two attestations sharing the same `(subject, activity_hash, data_hash)` but differing in `signer`, `source_hash`, or `source_type` are independent corroborations. Implementations MUST NOT treat them as duplicates.

Two attestations sharing `(subject, activity_hash)` but with different `data_hash` values are not necessarily contradictions. They are attestations about the same subject and activity type with different payload content.

Reference attestations (patterns such as `activity_type = correction | dispute | supersession` with `subject` pointing at another attestation's identifier) are addressed at the application layer.

### §2.6 Subject and witness_for fields

The **`subject`** field is 32 bytes interpreted as one of:

- an Ed25519 public key of another conforming signer;
- a content hash;
- an equivalent 32-byte identifier defined by the activity type's schema.

The interpretation MUST be documented in the activity type's schema.

The **`witness_for`** field is always present in the canonical byte sequence and carries one of:

- **32 zero bytes:** the attestation makes a first-order claim about the subject.
- **a 32-byte pubkey or content hash:** the attestation names another party's claim.

`witness_for` is a pointer, not a protocol operation. The referenced party does not sign anything as a consequence of being named. Applications that need cryptographic corroboration compose independent attestations per §2.5.2.

### §2.7 Timestamps and retention_hint

The **`signer_asserted_at`** field is a Unix epoch timestamp in seconds (int64) representing when the signer claims to have signed. Signers MUST NOT populate future timestamps beyond clock-skew tolerance; verifiers MAY reject attestations whose `signer_asserted_at` exceeds their local clock by more than 300 seconds.

**Substrate time is authoritative for recency and seniority.** Where an attestation has been notarized (Layer 4), the substrate-published timestamp (§5.1) is the authoritative time for questions of when the attestation entered the record. `signer_asserted_at` is the signer's claim; the substrate timestamp is the notary's observation. Verifiers reasoning about "when did X happen" for a notarized attestation MUST use the substrate timestamp.

The **`retention_hint`** field is an int64 encoding the signer's intent regarding payload storage duration:

- **`> 0`:** Unix epoch seconds after which the payload MAY be discarded by any party storing it.
- **`== 0`:** no expiry intent expressed; the implementation default applies.
- **`== -1`:** the signer intends the payload to be preserved indefinitely subject to storage limitations.

Retention is a hint, not a guarantee. The notary hash remains verifiable regardless of payload availability (§5.4).

### §2.8 Source license and off-chain metadata

Where an attestation's source carries license terms (Creative Commons on an ORCID record, an open-source license on a git commit, terms-of-service on a platform-recorded event), those terms bind the referenced content and any downstream use of it. License terms are NOT part of the canonical byte sequence.

Implementations MUST preserve license information alongside the attestation record and MUST propagate it to any verifier they disclose the attestation to. Implementations MUST NOT strip or replace license information when re-emitting attestations.

Implementations SHOULD store license information as a triple of `(source_hash, license_identifier, license_effective_range)` so that changes in source terms over time remain auditable.

`quality_flags` (a signer's annotations of known claim weaknesses) are also NOT signed over. Implementations SHOULD store them alongside the attestation and MUST NOT strip them.

Implementations SHOULD distinguish provenance produced at the original signing from provenance backfilled during migration via an off-chain `provenance_origin` flag (`original` or `backfilled`).

---

## §3 Layer 2: Signing

### §3.1 Canonical byte sequence for signing

Every signature covers the following byte sequence, in this order:

```
canonical_bytes =
      spec_version         (2 bytes, u16 little-endian)
   || signer               (32 bytes)
   || subject              (32 bytes)
   || activity_hash        (32 bytes)
   || data_hash            (32 bytes)
   || witness_for          (32 bytes)
   || source_hash          (32 bytes)
   || source_type          (2 bytes, u16 little-endian)
   || confidence           (2 bytes, u16 little-endian)
   || witnessing_depth     (1 byte, u8)
   || attestor_relationship (1 byte, u8)
   || signer_asserted_at   (8 bytes, int64 little-endian)
   || retention_hint       (8 bytes, int64 little-endian)
   || nonce                (32 bytes)
```

Total length: **248 bytes**.

Implementations MUST construct this exact byte sequence and MUST NOT include additional fields, framing, prefix bytes beyond `spec_version`, or version markers in the signed content. Additional metadata that an implementation stores or transports MUST NOT enter the canonical byte sequence.

The signature is `signature = Ed25519.sign(signer_privkey, canonical_bytes)` per §3.2, or the equivalent for a registered alternative algorithm (§3.3).

#### §3.1.1 spec_version marker

`spec_version` (u16 little-endian) is the first two bytes of the canonical byte sequence.

| Value | Version | Status |
|---|---|---|
| 1 | v0.1-preview | Deprecated. New attestations MUST NOT use this value. |
| 2 | v0.1-final | Deprecated. New attestations MUST NOT use this value. |
| 3 | v0.2 | Current. All new attestations MUST use this value. |

A verifier MUST inspect `spec_version` first and dispatch to the correct canonical-byte-sequence layout. Verifiers MUST reject attestations with unknown `spec_version` values as unverifiable, distinct from malformed, and SHOULD report the version-mismatch condition so implementers can distinguish "reader is behind the registry" from "attestation is corrupt."

Any change to the canonical byte layout advances `spec_version` and reserves a new integer here. Additive registry additions (new source_type values, new activity types) do NOT advance `spec_version`.

#### §3.1.2 Verification procedure

1. Read `spec_version` from the stored attestation. Dispatch to the correct canonical-byte-sequence layout.
2. Reconstruct `canonical_bytes` from the stored fields per that layout.
3. Verify `signature` against `canonical_bytes` using `signer` as the public key, per RFC 8032 §5.1.7.
4. If step 3 succeeds, the attestation is authentic: the holder of the `signer` private key produced this exact byte sequence.

Verification does not by itself establish that any given payload matches `data_hash` or that any given source matches `source_hash`. To verify the payload, the verifier MUST also compute `SHA-256(canonicalize(payload))` and compare against `data_hash`.

### §3.2 Ed25519 (mandatory)

Every conforming implementation MUST support Ed25519 per RFC 8032:

- Curve: Ed25519 (edwards25519), RFC 8032 §5.1.
- Encoding: 32-byte public key, 32-byte private seed, 64-byte signature.
- Signing message: the 248-byte `canonical_bytes` sequence, passed to `Ed25519.sign` without pre-hashing (PureEdDSA per RFC 8032 §5.1, not Ed25519ph).

Implementations MUST use PureEdDSA. Implementations MUST NOT use Ed25519ph.

### §3.3 Signature algorithm extension mechanism

Ed25519 is the sole algorithm in v0.2. Future versions MAY register additional algorithms in §9.5. Any implementation that encounters an attestation whose algorithm is not Ed25519 MUST reject it as unverifiable.

Adding an algorithm requires advancing `spec_version` and either introducing an algorithm identifier byte in the canonical sequence or specifying the algorithm implicitly per version. Implementations MUST NOT invent algorithm identifier bytes on their own.

### §3.4 Nonce (per-attestation uniqueness)

The **`nonce`** field is a 32-byte value ensuring that a signer producing two attestations with otherwise identical fields still produces two distinct signatures over two distinct canonical byte sequences.

The nonce is REQUIRED. It MUST satisfy:

- **Uniqueness per attestation.** Any two attestations produced by the same signer over the same subject with different payload content MUST have different nonces. This is required because §2.5.2 makes multiple same-signer, same-subject, same-activity attestations with different `data_hash` values first-class.
- **Unpredictability.** An attacker observing a stream of a signer's attestations SHOULD NOT be able to predict the nonce of the next attestation. Nonces MUST NOT be sequential counters visible to third parties.

**Reference derivation (non-normative).**

```
nonce = SHA-256(signer || subject || activity_hash || data_hash || salt_32)
```

where `salt_32` is a 32-byte value the signer holds and rotates. This derivation is deterministic for a given `(signer, subject, activity_hash, data_hash, salt_32)` tuple, produces distinct nonces for distinct payloads, and does not leak the pattern of the signer's activity as long as `salt_32` remains private.

Other derivations are acceptable provided they meet the two required properties. Purely random nonces (32 cryptographic-random bytes) satisfy both requirements trivially.

### §3.5 Key rotation (non-normative)

A signer's public key is expected to persist for the lifetime of the attestations produced under it. Rotation requires either:

- publishing a signed statement from the old key delegating to a new key (a pattern reserved for a future version); or
- accepting that attestations produced under the old key remain bound to the old key.

Implementations that hide the signing key from users (KMS, hardware-backed derivation, passkey PRF-derived seeds) MAY do so provided the resulting signatures verify per §3.1–§3.2.

---

## §4 Layer 3: Registry

Layer 3 defines what a signer is, how signer identity persists, and how signers withdraw prior statements. It deliberately does not define roles, affiliation, delegation, or hierarchy above the single-signer type; those are reserved for future versions (§4.5).

### §4.1 Signer identity

A signer is a single Ed25519 public key. There is no registration step, no directory service, and no central issuer. Any party holding an Ed25519 private key can produce a conforming attestation.

Implementations MUST NOT require signer pre-registration as a condition of accepting an attestation. Rate limiting, quota enforcement, and admission control at the transport layer (§6.5) are separate concerns.

The identity of a signer is exactly its public key. Any binding to a real-world person or organization is application-defined.

### §4.2 Persistent key semantics

Signer identity persists exactly as long as the signer's private key persists. Two attestations bearing the same `signer` value are, at the protocol layer, produced by the same signer regardless of when they were signed.

Implementations MUST NOT treat the same `signer` public key as two distinct signers under any circumstance. Implementations MAY offer signers a mechanism to label themselves but MUST NOT let such labels override the cryptographic identity of the key.

Signers MAY hold multiple distinct keys and use them in different contexts. Each key is a distinct signer at the protocol layer. Applications MAY offer off-chain grouping of keys ("these keys belong to the same real-world entity"); the protocol makes no such grouping.

### §4.3 Revocation by additive attestation

This specification has no on-chain mutation of prior attestations. Revocation is expressed by signing a new attestation that references the target.

**Required convention.** An attestation whose `activity_type` URI is `https://sworn.dev/v1/revocation` and whose `subject` is `SHA-256(target_canonical_bytes)` constitutes a revocation of the referenced target attestation by the signing party.

Signers MUST populate `subject` with the exact 32-byte SHA-256 of the target's canonical byte sequence (§3.1). Implementations MUST NOT substitute other identifiers (application UUIDs, substrate-specific PDA addresses, human-readable labels) for `subject` in a revocation.

Implementations MUST NOT delete, mutate, or otherwise alter the target attestation's record in response to a revocation. Both records remain durable. Verifiers walking the graph MUST see both the original attestation and any revocations of it, and MAY apply reader-side policy about how to weigh them.

**Who can revoke.** Only the original signer of an attestation can revoke it. A revocation whose signer differs from the original signer is not a revocation; it is a first-class attestation (a "dispute" or "counter-claim") that references the target but does not withdraw the target's signature.

**Dangling revocations.** A verifier MUST verify the revocation's signature per §3.1.2. A verifier MAY consider a revocation applied only to attestations whose canonical bytes hash to the value in `subject`. Revocations with no matching target are legitimate signed statements but have no target to apply to; verifiers SHOULD retain them (a matching target may surface later) but MUST NOT apply them to non-matching attestations.

**Additive only.** Every revocation is an additive record. Implementations MUST NOT provide a "hard delete" or "unpublish" path at Layer 4 (§5.4).

**Key rotation interaction.** Because v0.2 has no formal key rotation (§3.5), revocation requires the original signing key. A signer who has lost access to the key that produced an attestation cannot revoke that attestation in the v0.2 sense.

### §4.4 Standing as a graph

The accumulated record of attestations by and about a signer is a graph. It is not a score, a ranking, or a value. Implementations that compute derived signals over the graph MUST NOT present those signals as attestations.

Two implementations reading the same graph MAY compute different derived signals for legitimate reasons: different weightings across `witnessing_depth` values (§9.3), different treatment of low-`confidence` self-reports (§2.5.1), different decay curves for older attestations. This is expected and healthy. Cross-implementation portability applies to the graph (identical), not to derived signals (different).

Standing loss from revocation is not a spec-level mechanism. A revocation attestation (§4.3) is a fact in the graph; how much it decreases the referenced target's contribution to standing is a reader-side policy decision.

### §4.5 What is not in this layer

The following patterns exist in real recognition systems and are explicitly out of scope for v0.2:

- Two-layer witness/certifier roles.
- Affiliation records with revocation semantics distinct from attestation revocation.
- Delegated attestation.
- Multi-signature attestations. (Multiple independent signers about the same subject are already supported natively per §2.5.2.)
- Directory or discovery services.

Implementations MAY build any of these on top of v0.2's single-signer primitive as application conventions. Where they do, those conventions live in application documentation, not in this specification.

---

## §5 Layer 4: Notarization

Layer 4 defines how an attestation's cryptographic identity is committed to a public, tamper-evident ledger so that any verifier can confirm the attestation existed at a specific point in time.

**The notary in v0.2 is Solana Attestation Service (SAS).** The concrete binding, including credential and schema layout, PDA seed derivation, forbidden SAS instructions, and account data layout, is defined normatively in bindings/sas.md. The requirements in this section govern what any SAS-based notary MUST publish and MUST NOT permit; bindings/sas.md tells implementers how to satisfy them.

Non-SAS substrates do not offer Layer 4 conformance in v0.2. An implementation that produces signed attestations without notarizing them to SAS satisfies Layers 1 and 2 only; see bindings/postgres.md.

### §5.1 What the notary publishes

For each attestation, the notary MUST publish:

- **The `attestation_hash`**, defined as `SHA-256(canonical_bytes)` per §3.1.
- **A substrate-native creation timestamp** (Solana block time) monotonic within the substrate and independently readable by any verifier.

The notary MUST NOT include, in any part of its published record that is enumerable via a public substrate query (`getProgramAccounts` scans, indexed content lookups, memcmp filters on account content, or equivalents), the following fields:

- `signer`
- `subject`
- `activity_type` or `activity_hash`
- `source_hash`
- `witness_for`
- any deterministic encoding of the above

This is the discipline that keeps the notary from becoming a walkable dossier. The `attestation_hash` is opaque with respect to the fields it commits to; a scan of the substrate reveals which hashes have been published, not who signed what about whom. bindings/sas.md §3 defines the concrete SAS PDA derivation that satisfies this requirement.

**The notary does not attest to content.** Publishing an attestation's hash establishes that the hash existed at a known time. It does not endorse the truth of the attestation. Interpretations of the attestation live at Layer 3 (§4.4) and are reader-side concerns.

**Independent hash recomputation.** For a verifier to trust the published hash, the verifier MUST be able to independently reconstruct the canonical byte sequence (§3.1) from the attestation's stored fields, compute `SHA-256(canonical_bytes)`, and confirm the result matches what the notary published. A notary implementation that publishes a hash the verifier cannot independently reproduce from stored source material is NOT conforming.

**Notarization is not signing.** The party operating the notary MAY be a different party than the signer. Any party holding the canonical bytes and the signer's signature can compute the attestation's hash and publish it; the signature is not touched during notarization. This separation supports the "sign locally, notarize via a service" pattern.

### §5.2 Merkle batching

Some notary deployments batch attestation hashes into a Merkle tree and publish only the root as a cost optimization. This specification does not define a normative Merkle format at v0.2.

**Requirements for batching implementations.** An implementation that batches MUST anchor the resulting Merkle root as a distinct notary record satisfying §5.1 and MUST provide off-chain inclusion proofs to verifiers on request. Inclusion proofs verified against a batched root MUST resolve to the individual attestation's `SHA-256(canonical_bytes)` per §5.1's independent-recomputation rule.

Cross-notary Merkle interoperability (an inclusion proof from notary A confirming an attestation is anchored to notary B) is out of scope for v0.2. A normative Merkle format is reserved for v0.3.

### §5.3 Retention semantics

Layer 1's `retention_hint` field (§2.7) expresses the signer's intent regarding payload retention. Layer 4 governs the notary hash under the same hint.

**Required rule.** The notary hash MUST remain published for at least as long as the substrate maintains any record of the attestation. Substrates MUST NOT selectively expire notary hashes ahead of the substrate's own record-retention policy.

**Per-attestation heterogeneity.** A single notary substrate MAY carry attestations with different `retention_hint` values, and those attestations MAY have different off-chain payload availability at any time. A verifier MUST NOT assume all attestations in the same substrate share the same retention posture.

**Retention is not validity.** An attestation whose `retention_hint` has passed and whose payload has been reclaimed is still a valid attestation at Layer 2. Its signature verifies. What has changed is its disclosability (§6): the payload may no longer be retrievable, so a verifier may see only metadata.

### §5.4 Durability of the notary hash

Once published, the notary hash MUST NOT be revoked, mutated, or reversed by the notarization substrate under any protocol-visible circumstance. A verifier can rely on the invariant that a hash present today was present at its published timestamp and will remain present for as long as the substrate itself exists.

**Substrate-level failure modes** (chain reorganizations, ledger rollbacks) are out of protocol scope. Implementations SHOULD document what substrate-level guarantees they provide.

**Revocation is additive.** Per §4.3, revocation is expressed as a new attestation. The revocation is anchored per §5.1; the target attestation's notary hash is unaffected. Substrates MUST NOT interpret a revocation as license to delete or hide the target's notary hash.

**Substrate compromise.** If a substrate is discovered to have violated its published durability guarantees, attestations anchored to it lose the temporal grounding the substrate was providing. Signatures remain cryptographically valid; the "hash existed at time T" property collapses. Substrate-compromise recovery is not standardized in v0.2.

### §5.5 Off-chain payload storage

Payloads are NOT stored on the notary substrate by requirement of this specification. The notary record commits to `data_hash`; the payload lives off-chain, typically in the implementation's application database or an external content-addressed store.

Implementations MAY publish payload storage locations in off-chain metadata; the protocol does not require any specific storage model. What the protocol requires is that a verifier who obtains a payload can independently compute `SHA-256(canonicalize(payload))` and confirm equality with `data_hash` (§6.1).

### §5.6 Forbidden substrate operations

The SAS binding (bindings/sas.md §4) enumerates SAS instructions that a conforming implementation MUST NOT invoke on notarized attestations. Chief among them: the `tokenize` and `close` instructions. Invoking either would express a transfer or deletion the §5.4 durability rule forbids at the protocol layer.

---

## §6 Layer 5: Presentation

Layer 5 defines how third parties interact with attestations after Layer 4 has anchored them. The central discipline is *shown, not pulled*: a conforming implementation MUST support verification of attestations a caller already knows about, and MUST NOT support enumeration, discovery, or bulk export of attestations by properties of their subjects or signers. This is what keeps the attestation graph from becoming a surveillance database while preserving its utility as verifiable testimony.

### §6.1 Verification endpoint contract

A conforming implementation MUST expose a verification interface that, given an attestation identifier known to the caller, returns:

- the full canonical bytes of the attestation (or the primitive fields sufficient for the caller to reconstruct them per §3.1);
- the signature (64 bytes, per §3.2);
- the `activity_type` URI (from which `activity_hash` derives per §2.4);
- the notary-published `attestation_hash` (per §5.1) sufficient for the caller to confirm anchoring against the notary;
- the metadata required to interpret provenance fields (per §2.5).

The verification interface MUST NOT return the payload. Payload disclosure is a distinct operation per §6.2.

Transport is implementation-defined (HTTP, gRPC, CLI, direct substrate query, in-process library call).

**Independent verification.** Regardless of transport, a caller receiving the verification response MUST be able to independently reconstruct the canonical bytes, recompute `SHA-256(canonical_bytes)` and confirm equality with the notary-published `attestation_hash`, and verify the Ed25519 signature. An implementation whose response cannot be independently verified this way is NOT conforming, even if it returns a valid-looking status.

### §6.2 Two-call design: verify then disclose

Verification and payload disclosure are separate operations. Verification (§6.1) reveals who signed what class of claim about which subject, plus the hash commitment to the payload. Disclosure reveals the payload.

**Required separation.** Implementations MUST NOT return payload content from the verification interface. Implementations MUST require an explicit disclosure authorization (per §6.3) before returning payload bytes.

**Payload authenticity check on disclosure.** When an implementation returns a payload via disclosure, the caller MUST be able to compute `SHA-256(canonicalize(payload))` and confirm equality with `data_hash` from the verified canonical bytes. A disclosed payload whose recomputed hash does not match `data_hash` MUST be rejected as tampered, even if the disclosure token was valid.

### §6.3 Disclosure token semantics

A disclosure token authorizes exactly one retrieval of one attestation's payload. The token binds three properties: the specific attestation (by ID or hash), a time window during which the token is redeemable, and a single-use consumption guarantee.

**Required properties.**

- **Single-use by default.** A conforming implementation MUST redeem each disclosure token at most once. A second redemption attempt MUST fail with a distinct error. Implementations MAY offer explicitly-designated multi-use tokens as a separate token type; where they do, the multi-use property MUST be visible in the token's metadata.
- **Time-bounded.** Every token has an expiration. Implementations MUST reject expired tokens with a distinct error class from single-use exhaustion. Recommended range: minimum floor of 60 seconds, maximum ceiling of 7 days for single-use tokens.
- **Signer-authorized.** A disclosure token MUST be issued by proof of control of the attestation's signing key or by a mechanism the signer has explicitly authorized. Implementations MUST NOT permit unauthenticated parties to mint disclosure tokens for arbitrary attestations.
- **Domain-separated.** The bytes signed to authorize token issuance MUST NOT be substitutable for the canonical byte sequence of any attestation (§3.1) or the canonical form of any other operation defined by this specification. Implementations MUST use a domain separator that cannot collide with attestation canonical bytes; the reference domain separator is the literal string `sworn-disclosure-token-v1`.

### §6.4 Refused operations

A conforming implementation MUST NOT expose operations that enumerate attestations by properties of their subjects, signers, or payloads. The following MUST be refused (returned as errors, not silently unsupported):

- **List by subject.** A query returning all attestations naming a given subject.
- **List by signer.** A query returning all attestations produced by a given signer.
- **Bulk export.** A query returning attestations without a caller-supplied identifier.
- **Name or attribute search.** A query returning attestations whose payload content matches a search pattern.
- **Signer discovery.** A query returning signers matching a real-world identity, label, or attribute.

Implementations MUST return an explicit refusal (not treat these as absent features). The refusal is a first-class part of the presentation contract: an implementer testing conformance MUST observe the refusal to confirm the discipline is enforced.

**Rationale.** The design commitment is that the attestation graph is verifiable without being enumerable. A verifier holding an attestation identifier can confirm its authenticity; a party who does not hold an identifier cannot bulk-discover the graph's contents. The discipline is enforced at Layer 5 because it is the layer where callers meet the system. A raw substrate scan that bypassed this layer (for example, an unrestricted `getProgramAccounts` scan) is not itself a conforming presentation. §5.1's PDA-seed discipline exists so that even a raw substrate scan does not become a walkable index.

**Signer-scoped exceptions.** A signer authenticated to their own key MAY retrieve a list of their own attestations. This is self-service reflection, not enumeration by third parties. Implementations offering this MUST authenticate the request as coming from the signer's key.

**Application-scoped exceptions.** An implementation MAY expose enumeration within a bounded application context (all attestations attached to a specific event or resource the caller has independent access to) provided the scope is an application-layer resource identifier the caller must possess. Enumeration by properties of the attestation's signer or subject remains refused.

### §6.5 Rate limiting

Implementations MUST offer rate limiting on the verification and disclosure interfaces to prevent enumeration-by-timing (probing candidate identifiers to discover which resolve) and disclosure-endpoint abuse.

**Required posture.**

- The verification interface SHOULD carry a per-caller rate limit sufficient to prevent enumeration probing. Precise numeric thresholds are implementation-defined.
- The disclosure interface MUST carry a stricter rate limit than the verification interface, since disclosure returns payload bytes.
- Implementations MUST distinguish rate-limited responses from other error classes so callers can back off appropriately.
- Rate-limit tracking MUST NOT create a signer profile. Implementations MAY track per-IP or per-session request counts but MUST NOT correlate rate-limit state with signer identity.

---

## §7 Security considerations

The security surface is deliberately narrow. The protocol establishes that a specific signer produced a specific statement over specific canonical bytes and that the statement was anchored in a public durable substrate at a specific time. Everything else is reader-side interpretation or out of scope.

### §7.1 Sybil resistance is bounded

This specification provides no protocol-level mechanism to prevent one real-world party from operating multiple signer keys. Any Ed25519 keypair produces conforming attestations; the graph does not distinguish "one person with five keys" from "five people with one key each."

The graph is public. A cluster of keys signing back-and-forth attestations to each other is legible as such at the graph-analysis layer: density of mutual attestation, absence of external corroboration, and timing correlation between keys are all properties a verifier can compute. The `witnessing_depth` and `attestor_relationship` fields commit the signer's own claim about the epistemic depth of the witnessing act, making low-depth self-report clusters explicit rather than hidden.

This specification does not provide real-world identity verification, biometric uniqueness, proof-of-personhood, or KYC. Implementations that need such properties layer them above as application concerns.

### §7.2 Key compromise

A compromised signing key can produce attestations that are cryptographically valid but not authorized by the human or organization associated with the key. Because v0.2 has no formal key rotation (§3.5), the protocol offers no signature-layer distinction between "attestation produced before compromise" and "attestation produced after compromise."

Applications concerned with key compromise MUST layer their own detection above the protocol (behavioral analysis, monitoring for unexpected signing patterns, revocation attestations from the compromised key naming attestations to disregard).

### §7.3 Colluding attestation rings

Colluding signers can produce attestations that mutually corroborate false claims. This specification does not detect such collusion at the protocol layer; the graph-analysis approach in §7.1 is a reader-side heuristic, not a spec-enforced property.

Applications MAY require attestations from signers with independent standing (measured by graph position, external-source provenance, or off-chain identity binding) as a policy layer above the protocol. Such policies are application-defined and SHOULD be documented alongside the standing-conversion transparency requirements of the applications themselves.

### §7.4 Payload availability versus hash durability

The notary hash is durable (§5.4). The payload is not; retention hints (§2.7) may allow the payload to be discarded. Verifiers MUST distinguish "the signature verifies but the payload has been reclaimed" from "the signature does not verify." The former is a valid attestation with limited disclosability; the latter is an invalid attestation.

Applications relying on future payload retrieval SHOULD either use `retention_hint = -1` or maintain their own payload storage independent of any single implementation's retention policy.

### §7.5 Timestamp trust

The signer's `signer_asserted_at` is a claim, not a proof. A signer can produce past-dated attestations, though verifiers MUST reject future-dated ones beyond clock-skew tolerance (§2.7). For attestations that have been notarized, the substrate-published timestamp is authoritative (§2.7, §5.1). For questions of "when did the signer make this claim," verifiers MUST prefer the substrate timestamp over `signer_asserted_at`.

---

## §8 Privacy considerations

Privacy properties emerge from a specific split: the facts of attestation (who signed, when, what class of claim, anchored where) are the signer's own responsibility to control at attestation time; the content of the payload is not published on the notary substrate and reaches verifiers only through the disclosure discipline of Layer 5 (§6). This section names what the split provides, what it does not, and where implementers carry privacy load the protocol does not carry.

### §8.1 Public verification, private payloads

The canonical byte sequence commits to `data_hash`, a SHA-256 of the canonicalized payload. The payload itself is never part of the canonical bytes. This is the primitive that allows public verification of private content.

**Required.** Implementations MUST NOT publish attestation payloads to the notary substrate as a side effect of anchoring the hash. Payloads live off-chain; they reach verifiers only through Layer 5's disclosure endpoints.

**What is public regardless.** The following are readable by anyone with access to the reconstructible canonical bytes: `signer` pubkey, `subject`, `activity_type`, `data_hash`, `source_hash` and `source_type`, timestamps, `witness_for`, `nonce`, and `signature`. Implementers designing subject or provenance schemas MUST assume these fields are as public as any store from which they might be retrieved.

Note the interaction with §5.1: the notary substrate itself does not expose these fields via a walkable index. But any party holding the canonical bytes (through legitimate presentation or through a payload disclosure) learns all of them.

**Subject design.** If an activity type places a real-world identifier directly into `subject` (a bare email, a legal name), that identifier is present in the canonical bytes and therefore learnable by any legitimate holder of them. Schemas that need subject privacy SHOULD use a content hash of an identifier plus a per-subject salt held by the signer, not the identifier itself.

### §8.2 Signer-authorized disclosure

Layer 5's two-call design (§6.2, §6.3) is the mechanism by which a verifier gains access to an attestation's payload. Verification requires no signer involvement; disclosure requires a signer-authorized disclosure token.

**Required.** Implementations MUST NOT expose an unauthenticated payload-retrieval path indexed by attestation identifier. Any endpoint returning payload content MUST require a valid disclosure token per §6.3.

**Delegation.** Implementations MAY offer signer-delegated disclosure policies (a signer grants a third party the right to mint tokens for a defined subset of the signer's attestations) provided such delegation is itself an attestation. Silent delegation is prohibited.

**Subject-as-signer.** When the signer is also the subject (`attestor_relationship = self`, §9.4), signer-authorized disclosure is equivalent to subject-authorized disclosure. Cases where signer and subject differ are addressed by activity-type schemas; this specification does not enforce subject-consent semantics separately from signer authorization.

### §8.3 Right to be forgotten and immutable hashes

The tension between deletion rights (GDPR Article 17 and equivalents) and cryptographic immutability is real and not fully resolvable by protocol design. This section names how the specification divides the surface so the resolvable parts can be resolved and the unresolvable parts are legible as such.

**What this specification makes tractable.** Because payloads are off-chain and subject to retention hints (§2.7, §5.5), the content carrying personal data can be discarded by the parties retaining it. A subject requesting erasure can be honored by removing the payload from every retention source under the implementation's control. The notary hash remains but reveals nothing about the payload beyond that some 32-byte value was committed.

**What remains regardless.** The canonical bytes are re-derivable by anyone who ever held them. `signer`, `subject`, `activity_type`, `data_hash`, provenance fields, and timestamps are inside the signed bytes. If the `subject` field carries a real-world identifier directly, that identifier is embedded in any legitimately-held canonical bytes and is not erasable without invalidating the signature.

**Notary-side.** The notary substrate publishes only `attestation_hash` and a timestamp under the §5.1 discipline. The substrate itself does not carry a walkable copy of the canonical bytes. But the substrate does confirm that an attestation with a particular `attestation_hash` was published at a particular time, and any party who obtains the canonical bytes elsewhere can prove they match the published hash.

**Required disclosure to signers and subjects.** Implementations that accept personal data into conforming attestations MUST clearly disclose, before signing, which fields will be part of the signed bytes (durable and re-derivable by any holder) and which will be retention-controllable (in the payload). Silent conflation is a privacy failure.

**Metadata-as-personal-data.** In some legal readings, surviving metadata (fact of signing, activity type, timestamp) constitutes personal data. This specification provides no protocol mechanism to erase this surviving metadata. Implementations for which this is unacceptable MUST either use conforming attestations only for content whose metadata is not itself personal data, or not use this specification for that content class.

### §8.4 Pseudonymity of signers

Signers are 32-byte Ed25519 public keys. Any mapping between a public key and a real-world person or organization is application-defined.

**Required.** Implementations MUST NOT require signers to be linked to a verified real-world identity as a condition of accepting their attestations.

**Pseudonymity is not anonymity.** Standing accumulates to a public key. A pseudonymous signer who accumulates a large body of attestations has, from a reader's perspective, a legible history under that pseudonym. A reader who links `signer` pubkey P to a real-world entity by other means (traffic analysis, off-chain leaks, self-disclosure) can then attribute all of P's attestation history to that entity.

Signers who want unlinkability across attestations MUST use a distinct key per attestation, accepting the loss of accumulated standing under any one pseudonym. The protocol makes both patterns possible and enforces neither.

**Not provided by the protocol.** No mixnet for the notarization transaction. No zero-knowledge proofs of attestation properties in v0.2. No forward secrecy for disclosed payloads (once a payload is disclosed under a valid token, the recipient can retain, redistribute, or leak it).

---

## §9 Registries

### §9.1 Activity type namespace registry

This registry lists namespaces reserved for use with activity type URIs. Registration is descriptive: an implementation is free to use any well-formed URI as an activity type (§2.2), and this section documents namespaces already in use so implementers can align without collision.

| Namespace prefix | Owner / source | Purpose |
|---|---|---|
| `https://sworn.dev/v1/` | This specification's own namespace | Well-known types defined here (see §4.3 revocation type). |
| `https://credit.niso.org/contributor-roles/` | NISO (Z39.104-2022) | CRediT (Contributor Roles Taxonomy). See §9.1.1. |

Non-reserved namespaces (any well-formed URI a signer chooses to use) remain valid activity types.

**Registered types under `https://sworn.dev/v1/`.**

| URI | Purpose | Payload schema |
|---|---|---|
| `https://sworn.dev/v1/revocation` | Revoke a prior attestation by the same signer (§4.3) | `subject` MUST be `SHA-256(target_canonical_bytes)`. Payload MAY carry a human-readable reason. |

Additions to the `sworn.dev/v1/` namespace are additive and do not advance `spec_version`.

#### §9.1.1 CRediT (Contributor Roles Taxonomy)

CRediT is a fourteen-role vocabulary maintained by NISO as ANSI/NISO Z39.104-2022. This specification registers the CRediT namespace so attestations recognizing research contributions can share a widely-adopted vocabulary.

**URI pattern.** Each CRediT role maps to a URI of the form `https://credit.niso.org/contributor-roles/<slug>/`, where `<slug>` is a lowercase kebab-case rendering of the role name. The URIs include the `https://` scheme and the trailing slash as published by NISO.

**Fourteen roles.**

| Slug | Role | Definition (summary) |
|---|---|---|
| `conceptualization` | Conceptualization | Ideas; formulation of overarching research goals and aims. |
| `data-curation` | Data curation | Management activities to annotate, scrub, and maintain research data. |
| `formal-analysis` | Formal analysis | Statistical, mathematical, or computational analysis of study data. |
| `funding-acquisition` | Funding acquisition | Acquisition of financial support for the project. |
| `investigation` | Investigation | Conducting the research process; performing experiments or evidence collection. |
| `methodology` | Methodology | Development or design of methodology; creation of models. |
| `project-administration` | Project administration | Management and coordination of the research activity. |
| `resources` | Resources | Provision of study materials, reagents, patients, samples, compute resources. |
| `software` | Software | Programming, software development, algorithm design, implementation, testing. |
| `supervision` | Supervision | Oversight and leadership responsibility for the research activity. |
| `validation` | Validation | Verification of reproducibility of results and other experimental outputs. |
| `visualization` | Visualization | Preparation, creation, and presentation of published work, specifically visualization. |
| `writing-original-draft` | Writing, original draft | Preparation and presentation of the initial draft. |
| `writing-review-editing` | Writing, review & editing | Critical review, commentary, or revision of published work. |

**Degree of contribution.** CRediT allows an optional degree qualifier of `lead`, `equal`, or `supporting`. Implementations SHOULD carry the degree in the attestation payload (not in the URI). Example: `"contribution_degree": "lead"`.

The CRediT taxonomy itself is owned and maintained by NISO. Implementations using these URIs are consuming CRediT, not extending it.

### §9.2 source_type registry

The `source_type` field (§2.5) is a u16 whose registered values are given below. Integer positions are stable per §1.5. Future additions append at the next unused integer. Additions are additive and do NOT advance `spec_version`.

For each value, this registry specifies the canonical string label, what the source represents, and how implementations MUST derive `source_hash` for that source_type. Cross-implementation source-identity matching depends on all implementations agreeing on the derivation procedure.

| # | Slug | Description | source_hash canonicalization |
|---|---|---|---|
| 0 | `unknown` | Source not classified. | `source_hash` MUST be 32 zero bytes. |
| 1 | `self_reported` | The signer is asserting a fact about themselves with no external source. | `source_hash` MUST be 32 zero bytes. |
| 2 | `orcid` | Sourced from an ORCID record. | MUST: `SHA-256` of the 19-character ORCID identifier as ASCII, upper-hyphen form (e.g., `0000-0002-1825-0097`). No scheme, no host, no trailing whitespace. |
| 3 | `doi` | Sourced from a DOI-resolvable publication. | MUST: `SHA-256` of the bare DOI in lowercase ASCII (e.g., `10.1234/example.5678`). No `doi.org/`, no scheme, no fragment. |
| 4 | `openalex` | Sourced from an OpenAlex record. | MUST: `SHA-256` of the uppercase OpenAlex ID with type prefix (e.g., `W1234567890`). No scheme, no host. |
| 5 | `git_commit` | Sourced from a specific git commit. | MUST: `SHA-256` of the full 40-character lowercase hexadecimal commit SHA. No repo prefix, no branch context. |
| 6 | `rss_parsed` | Machine-extracted from an RSS/Atom feed. | SHOULD: `SHA-256` of the item's `<guid>` or `<atom:id>` value as UTF-8. Where the feed provides neither, `SHA-256` of the item's canonical URL after NFC normalization. |
| 7 | `open_source_project` | Sourced from a project's declared authorship (CITATION.cff, package metadata, etc.). | SHOULD: `SHA-256` of the primary repository URL (lowercase scheme+host+path). |
| 8 | `coordinator_confirmed` | A coordinator role in the community affirmed the claim. | SHOULD: `SHA-256(coordinator_signer_pubkey || confirmation_timestamp_int64_le)`. Deterministic within an implementation. |
| 9 | `peer_witnessed` | A peer directly witnessed the claimed activity. | MUST: `SHA-256` of the peer's 32-byte signer pubkey. |
| 10 | `computed` | The claim was derived algorithmically from other data. | SHOULD: `SHA-256(algorithm_identifier_utf8 || canonicalized_inputs_bytes)`. |
| 11 | `system_observed` | The claim is a system-observed fact (attendance record, transaction confirmation, computed platform stat). | SHOULD: `SHA-256(platform_identifier_utf8 || event_id_utf8)`. |
| 12 | `regulatory_filing` | Sourced from a legally-mandated public disclosure. | MUST: `SHA-256(filing_type_identifier_utf8 || filing_identifier_utf8)` (e.g., `990:EIN:12345:2024` for an IRS 990). |
| 13 | `community_curated_db` | Sourced from a community-edited database with revision history. | MUST: `SHA-256` of the canonical entity URL after NFC normalization (e.g., `https://musicbrainz.org/artist/<mbid>`). |
| 14 | `external_sworn_attestation` | References another conforming attestation (federation, cross-implementation graph). | MUST: `SHA-256` of the referenced attestation's canonical byte sequence, per §3.1 of the version that attestation was signed under. |
| 15 | `oauth_authenticated` | The signer's identity was verified by a third-party OAuth provider at attestation time. See §9.2.1. | MUST: `SHA-256("oauth:" \|\| provider_name_utf8 \|\| ":" \|\| provider_user_id_utf8)`. |

**Enum evolution.** Verifiers encountering a `source_type` value they do not recognize MUST report a version-mismatch condition distinct from malformed-attestation. Verifiers MUST NOT interpret unknown source_type values as `unknown` (0); the enum is exhaustive at each version.

#### §9.2.1 OAuth provider names

`source_type = 15` (`oauth_authenticated`) requires a registered `provider_name` short string.

| provider_name | Provider | provider_user_id semantics |
|---|---|---|
| `github` | GitHub | Numeric user ID (e.g., `12345678`), NOT username. |
| `linkedin` | LinkedIn | LinkedIn member ID from the OAuth response. |
| `google` | Google | `sub` claim from the OAuth ID token. |
| `orcid` | ORCID | 19-character ORCID identifier in upper-hyphen form. Distinct from `source_type = 2`: this means "authenticated via ORCID's OAuth flow at attestation time"; source_type 2 means "the underlying data came from an ORCID record." |

**Confidence guidance (non-normative).** OAuth-mediated attestation verifies control of a third-party account, not human identity of its owner. `10000` (100%) is inappropriate for any OAuth path. A reasonable floor across all OAuth providers is `7000-8000` (70-80%).

### §9.3 witnessing_depth registry

The `witnessing_depth` field (§2.5) is a u8. Integer positions are stable per §1.5.

| # | Slug | Meaning |
|---|---|---|
| 0 | `unspecified` | Depth not classified. Legitimate for backfilled records or where the signer chooses not to characterize. |
| 1 | `physically_observed` | The signer was present when the claimed activity occurred and directly witnessed it. |
| 2 | `reviewed_artifacts` | The signer inspected outputs (code, papers, receipts, records) after the fact. |
| 3 | `ui_confirmed` | The signer confirmed the claim through a user-interface action (e.g., pressed a button) without deeper inspection. |
| 4 | `computed_match` | The signer is a machine process that produced the claim by matching against reference data. |
| 5 | `self_asserted` | The subject and signer are the same party; no separate witnessing occurred. |

### §9.4 attestor_relationship registry

The `attestor_relationship` field (§2.5) is a u8. Integer positions are stable per §1.5.

| # | Slug | Meaning |
|---|---|---|
| 0 | `unknown` | Relationship not classified. Legitimate for backfilled records. |
| 1 | `self` | The signer and subject are the same party. |
| 2 | `coordinator` | The signer holds a coordinator or moderator role in the subject's community context at signing time. |
| 3 | `peer` | The signer is a peer in the subject's community context. |
| 4 | `mentor` | The signer is a mentor, supervisor, or teacher to the subject. |
| 5 | `unaffiliated` | The signer has no prior relationship to the subject. |
| 6 | `institution` | The signer is an institutional or automated entity attesting on behalf of a platform, employer, or ingestion pipeline. |

### §9.5 Signature algorithm registry

Ed25519 is the sole registered signature algorithm in v0.2. Future versions may register additional algorithms per §3.3.

### §9.6 Notarization substrate registry

In v0.2 the sole registered notarization substrate is Solana Attestation Service (SAS) as defined in bindings/sas.md. Future versions may formalize this registry to accommodate additional substrates.

---

## §10 Conformance

Conformance to this specification is defined by what an implementation can produce, consume, and refuse.

### §10.1 Conformance levels

An implementation MAY conform at one of three levels. Each higher level entails the lower ones.

**Level 1: Verifier.** An implementation that can consume, validate, and reason about attestations produced by any other conforming implementation. A verifier MUST:

- Reconstruct the 248-byte canonical byte sequence (§3.1) from a stored attestation's fields.
- Verify Ed25519 signatures per §3.2 (PureEdDSA, no pre-hashing).
- Recompute `SHA-256(canonicalize(payload)) == data_hash` per §2.3 and §2.4 when a payload is present.
- Recompute `SHA-256(canonical source identifier)` per §9.2's per-source-type canonicalization when checking source integrity.
- Recompute `SHA-256(target_canonical_bytes)` when interpreting revocation references per §4.3.
- Reject attestations whose `spec_version` marker (§3.1) is unknown to the implementation, distinguishing that failure from signature invalidity.
- Refuse enumeration and bulk-retrieval operations at any Layer 5 endpoint per §6.4.

A verifier MAY be embedded in a browser, a library, a command-line tool, or a service.

**Level 2: Signer.** A verifier that also produces new attestations. A signer MUST additionally:

- Construct canonical byte sequences that pass byte-for-byte verification against §10.4's reference vectors.
- Generate nonces per §3.4's uniqueness and unpredictability rules.
- Self-verify its own signatures before publishing them, catching client-side signing bugs before they propagate.
- Populate provenance fields per §2.5, including the zero-hash rule for sourceless `source_type` values (§2.4) and the range and enum constraints (§9.2 through §9.4).

**Level 3: Notarizer.** A signer that also anchors attestations to Solana Attestation Service per bindings/sas.md. A notarizer MUST additionally:

- Publish the notary record satisfying §5.1's requirements, using the PDA derivation from bindings/sas.md §3.
- Not invoke the forbidden SAS instructions listed in bindings/sas.md §4.
- Provide independent hash recomputation as required by §5.1.

### §10.2 Interoperability tests

Every level MUST pass the golden-vector suite at fixtures/attestations/. Vectors are byte-exact: an implementation whose serializer produces different bytes than the vectors for the same inputs is not conforming.

Notarizers MUST additionally pass the SAS binding test suite at fixtures/tests/sas/. Signers and verifiers MUST additionally pass the HTTP conformance test suite at fixtures/tests/http/ against their Layer 5 endpoint.

### §10.3 Registration process

There is no central registry of conforming implementations in v0.2. An implementation MAY self-declare conformance by:

- Passing the golden-vector suite for its declared level.
- Publishing the passing test output alongside the implementation's source repository.
- Documenting its Layer 4 substrate binding (for notarizers) or its Level 1-2 partial-conformance status (for signers-without-notarization).

A formal registration process is reserved for a future version once implementer adoption justifies it.

### §10.4 Reference test vectors

The canonical set of test vectors lives at fixtures/attestations/v0.2/vectors.json in the specification repository. Each vector specifies:

- inputs for constructing an attestation;
- the expected canonical byte sequence in hex;
- the expected Ed25519 signature under a specified test key;
- the expected `attestation_hash` (SHA-256 of canonical bytes).

Implementations MUST produce byte-identical canonical sequences and hash values for the specified inputs. An implementation whose output differs from any vector is not conforming.

Vectors cover the meaningful edge cases: sourceless attestations (source_type ∈ {0, 1}), sourced attestations across each registered source_type, revocations, and attestations with each combination of witness_for-populated and unpopulated states.

---

## Appendix A: Publication history

**v0.2 (this document).** Substantial revision from v0.1-final following external review. The canonical byte sequence advances `spec_version` from 2 to 3. The rename `created_at` to `signer_asserted_at` reflects the field's semantics as the signer's claim rather than an authoritative timestamp. The Layer 4 notary requirements now include the non-walkability discipline of §5.1 and require the SAS binding of bindings/sas.md for full conformance. The `sworn.dev/v1/revocation` activity type is now registered rather than reserved. §1.5 (previous non-transferability firewall) is retired; §1.4 lists non-goals. All references to specific applications built on the specification move to PRIMER.md. The SWORN name is retired from prose; see PRIMER for historical context.

**v0.1-final.** Added five provenance fields (`source_hash`, `source_type`, `confidence`, `witnessing_depth`, `attestor_relationship`) to the canonical byte sequence and prefixed the sequence with an explicit `spec_version` marker. Canonical byte length grew from 208 bytes to 248 bytes. Under review; not published for external signing.

**v0.1-preview.** Initial draft. Under review; not published for external signing.

## Appendix B: Glossary

**Activity type.** URI naming the class of claim being made.

**Attestation.** Signed statement by one party about another party or artifact.

**Attestation hash.** `SHA-256(canonical_bytes)`. The identifier by which the notary publishes an attestation and by which revocations reference their targets.

**Canonical bytes.** The 248-byte sequence defined in §3.1 that is passed to Ed25519.sign.

**Canonicalization.** Deterministic serialization per RFC 8785 for JSON payloads (§2.3) or per §9.2 for source identifiers.

**Ed25519.** The signature algorithm specified in RFC 8032, used in PureEdDSA form throughout this specification.

**Notarization substrate.** The public tamper-evident ledger where attestation hashes are committed. Solana Attestation Service in v0.2.

**Payload.** Semantic content of the attestation; JSON object canonicalized per §2.3 and hashed as `data_hash`.

**Provenance.** The signer's claim about origin and method, captured by `source_type`, `source_hash`, `confidence`, `witnessing_depth`, `attestor_relationship`.

**Signer.** The Ed25519 public key that produced an attestation's signature.

**Standing.** The accumulated record of attestations by and about a signer, treated as a graph rather than as a value.

**Subject.** The entity being attested about.

## Appendix C: Substrate bindings

The following bindings live in the specification repository. Bindings other than the SAS binding are informative and do not provide Layer 4 conformance.

- **bindings/sas.md** (Solana Attestation Service). Normative. Defines credential and schema layout, PDA seed derivation, forbidden instructions, and account data layout. Required for Level 3 (Notarizer) conformance.
- **bindings/postgres.md** (Postgres via [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres)). Informative. Documents a Layer 1 + Layer 2 partial-conformance implementation that produces signed attestations without publishing them to a substrate.

An implementation MAY use another substrate for Layers 1 and 2 storage while still producing conforming attestations; such an implementation is not a Level 3 Notarizer under v0.2.
