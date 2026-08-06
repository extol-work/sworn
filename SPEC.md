# SWORN Specification v0.1-final

**Status:** DRAFT. Not stable. Not soliciting external review until the reference implementation exercises the text and surfaces the gaps.

§1 through §3 have required text drafted against a working reference implementation. §4 through §10 are section outlines pending drafts that will be written from working code, not from prose.

**Notation:** The words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are to be interpreted as described in RFC 2119.

---

## Preface: v0.1-preview to v0.1-final

This specification advanced from v0.1-preview to v0.1-final during the pre-review window, following reviewer discussion of provenance treatment at Layer 1. The two revisions differ substantively: v0.1-final adds five provenance fields to the canonical byte sequence (`source_hash`, `source_type`, `confidence`, `witnessing_depth`, `attestor_relationship`) and prefixes the sequence with an explicit `spec_version` marker. The canonical byte length grew from 208 bytes to 248 bytes.

Because no external reviewer had attested to the v0.1-preview canonical bytes, the transition preserves the RFC's posture that accumulating corroboration, not committee ratification, governs how versions advance. The v0.1-preview text remains in the repository history and its canonical bytes are not conforming to v0.1-final. A signature over v0.1-preview bytes is not a signature over v0.1-final: any implementation that participated in pre-review signing knows their signatures require re-issuance under v0.1-final semantics to be conforming.

This transition is spec hygiene, not walk-back. Publishing a preview, taking review, restructuring cleanly before external attestation begins is the RFC posture working exactly as intended.

---

## §1 Overview

### §1.1 Purpose

SWORN specifies how testimony (the small, corroborable claims that used to live in guilds, unions, and professional societies) can be captured cryptographically so that anyone can verify it later, from anywhere, without needing to trust the platform where it was originally recorded.

A conforming SWORN attestation has four properties:

- **Durable.** The attestation's hash is committed to a public, tamper-evident ledger.
- **Portable.** The attestation carries its own signature and provenance; no lookup against the platform of origin is required to verify it.
- **Independently verifiable.** Any party with the attestation can verify its signature without permissioned access.
- **Sourceable.** The attestation names its own origin (via `source_hash` and `source_type`), so a verifier can reason about the claim's basis, not only about the signer's identity.

A signature tells you who staked their name. Provenance tells you what they staked it on. The two together are what separates a witnessed claim from a credit-report fact you cannot audit.

This document defines version 0.1 of the specification. It is deliberately minimal. Multi-signer role patterns, delegation, and cross-chain notarization are reserved for future versions (see [OPEN_QUESTIONS.md](./OPEN_QUESTIONS.md)).

### §1.2 Terminology

**Attestation.** A signed statement by one party (the *signer*) about another party or artifact (the *subject*). The statement is captured as a fixed-length hash of a payload, together with metadata that describes what class of statement is being made, when, and from what source.

**Signer.** The public key that produced an attestation's signature. Standing accrues to the signer, not to the person or organization that operates it. Signers are persistent (the same key over time), but the mapping between a signer and any real-world identity is implementation-defined.

**Subject.** The entity being attested about. May be another SWORN signer, an arbitrary public key, or a content hash. Subject semantics are activity-type-defined.

**Payload.** The semantic content of the attestation. Free-form structured data whose shape is determined by the activity type. The payload is NOT committed on-chain; only its hash is (§2.4, §5.1).

**Activity type.** A URI naming the class of claim being made (e.g., `work.extol.attestation/v1/contribution`, `sworn.dev/validator/v1/block-building-disclosure`). Activity types define the payload schema.

**Provenance.** The signer's claim about the origin of an attestation, captured by two orthogonal axes: *what source* (via `source_hash` and `source_type`) and *how the witnessing occurred* (via `witnessing_depth` and `attestor_relationship`). Provenance is signed content; it commits the signer's claim about origin as part of the attestation. Provenance is the signer's assertion, not the verifier's guarantee (§2.5).

**Notarization substrate.** The public, tamper-evident ledger where attestation hashes are committed. May be a blockchain, a git-anchored append-only log, a certificate transparency log, or any equivalent system. Substrate choice is implementation-defined (§5.1); interoperability is achieved through the substrate registry (§9.6).

**Standing.** The accumulated history of attestations to or by a signer. Standing is NOT a score; it is a graph. Standing MUST NOT be transferable (§1.5). Any conversion of standing into transferable value is a product-layer concern subject to the transparency requirement in §1.5.

**Conforming implementation.** Any software that produces, stores, and verifies attestations per this specification. See §10 for conformance criteria.

### §1.3 Layer model

SWORN is organized in five layers:

- **Layer 1, Testimony (§2):** the structure of an attestation record, including its provenance
- **Layer 2, Signing (§3):** how an attestation is bound to a signer
- **Layer 3, Registry (§4):** signer identity semantics and revocation
- **Layer 4, Notarization (§5):** how attestation hashes are committed to a public ledger
- **Layer 5, Presentation (§6):** how third parties verify attestations

Layers may be implemented independently, but a conforming implementation MUST implement all five. Implementations MAY use any substrate for Layer 4 (§5.1) and any presentation contract that satisfies §6's constraints.

### §1.4 Notational conventions

The words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in this document are to be interpreted as described in RFC 2119.

Byte-level fields are described in little-endian byte order unless explicitly noted. All hashes are SHA-256 (per FIPS 180-4) unless otherwise specified. All signatures are Ed25519 (per RFC 8032, §3.2) unless a registered alternative algorithm is used (§3.3).

Public keys are 32-byte Ed25519 encoded points. Signatures are 64 bytes. Byte concatenation is denoted `||`.

Integer enum values are stable: once a value is assigned an integer position in the registries at §9.2 through §9.4, that position does not change. Renames of the string label are permitted; renumbering is not.

### §1.5 Non-transferability firewall (required)

> Attestations MUST NOT be transferable between keys. Implementations MUST NOT wrap attestations in fungible tokens.
>
> Any conversion of attestation-derived standing into transferable value (governance weight, token allocations, service tiers, tradeable badges, or equivalent) MUST be publicly documented, versioned, and referenceable by URI. The documentation MUST specify:
>
> - **(a)** the inputs from the attestation graph consumed by the conversion,
> - **(b)** the output value form,
> - **(c)** any subjective or off-chain inputs to the function, and
> - **(d)** the effective date of the current version.

**Rationale.** Attestations derive their value from being witnessed statements, not tokens. A witness stakes their standing on the truth of a claim; if standing could be sold, the stake disappears and the claim becomes noise. The non-transferability requirement is not a philosophical preference. It is what makes the entire graph legible. The transparency requirement makes the constraint enforceable: without it, an implementation could convert standing into transferable value opaquely and preserve the letter of §1.5 while violating its spirit.

Implementations MAY produce derived signals from the attestation graph (rankings, scores, tiers, indices) and MAY exchange those signals for value, provided (a) the derivation function is documented per this section and (b) the derived signal itself is not presented as an attestation.

---

## §2 Layer 1: Testimony

### §2.1 Attestation record structure

An attestation record is a tuple with the following fields:

| Field | Type | Length | Description |
|---|---|---|---|
| `spec_version` | u16 little-endian | 2 bytes | The specification version this attestation is signed against. See §3.1. |
| `signer` | Ed25519 public key | 32 bytes | Produced the signature. See §3. |
| `subject` | pubkey or content hash | 32 bytes | Entity being attested about. See §2.6. |
| `activity_type` | URI (UTF-8) | variable | Names the class of claim. See §2.2. |
| `data_hash` | SHA-256 | 32 bytes | Hash of the canonical payload. See §2.3–§2.4. |
| `witness_for` | pubkey OR 32 zero bytes | 32 bytes | Optional endorsement target. See §2.6. |
| `source_hash` | SHA-256 or 32 zero bytes | 32 bytes | Hash of the canonical source identifier. See §2.4–§2.5. |
| `source_type` | u16 little-endian | 2 bytes | The kind of source. See §2.5, §9.2. |
| `confidence` | u16 little-endian | 2 bytes | Signer's confidence estimate, 0–10000 bps. See §2.5. |
| `witnessing_depth` | u8 | 1 byte | The epistemic depth of the witnessing act. See §2.5, §9.3. |
| `attestor_relationship` | u8 | 1 byte | Signer's relationship to the subject. See §2.5, §9.4. |
| `created_at` | int64 Unix seconds | 8 bytes | When the attestation was signed. See §2.7. |
| `retention_hint` | int64 | 8 bytes | Payload retention hint. See §2.7. |
| `nonce` | opaque 32 bytes | 32 bytes | Deterministic uniqueness. See §3.4. |
| `signature` | Ed25519 signature | 64 bytes | Over canonical bytes. See §3.1. |

This tuple is the canonical form of an attestation. Serialization for storage and transport is implementation-defined; a conforming implementation MUST be able to reconstruct the canonical byte sequence (§3.1) from stored data in order to verify the signature.

Not signed over, but required for a complete storage record: `payload` (whose hash is `data_hash`) and any off-chain metadata annotations (see §2.8).

### §2.2 Activity type namespacing

An `activity_type` is a URI naming the class of claim being made. The URI MUST be:

- absolute (not relative);
- resolvable, in principle, to a schema document describing the payload structure; and
- stable, meaning implementations MUST NOT change the meaning of an existing URI. A schema evolution requires a new URI (typically a version suffix, e.g., `.../v2/contribution`).

Examples of well-formed activity types:

- `work.extol.attestation/v1/contribution`
- `sworn.dev/validator/v1/block-building-disclosure`
- `https://schemas.example.org/hr1/statement-of-service`

Namespace prefixes MAY be reverse-DNS style (`org.example.foo`) or URL-style (`https://example.org/foo`). Both are canonical URIs after Unicode NFC normalization and produce identical signing behavior (§2.4).

**Extension mechanism.** Any party MAY define a new activity type by publishing a schema at the URI. There is no central registry for activity types in v0.1; a signer's willingness to use an activity type and a verifier's willingness to interpret it are the coordination mechanism. Reserved namespaces documented in §9.1 exist to promote interoperability, not to constrain implementers.

**Adopting established vocabularies.** Where an established vocabulary already exists for a domain, implementations SHOULD adopt it rather than mint a parallel namespace. Established vocabularies enable cross-implementation legibility without requiring coordination between implementers. See §9.1 for reserved namespaces, including the CRediT contributor role taxonomy (research contributions), Extol's application vocabulary (community activity), and space for W3C Verifiable Credentials and Open Badges to be registered as adopters emerge.

**Extol namespace grandfathering (informative).** For continuity with the first production adopter (Extol, Inc.), the following activity types are reserved under `work.extol.attestation/v1/`:

`participation`, `contribution`, `journal-digest`, `vote-participation`, `proposal-creation`, `event-organizing`, `group-founding`, `delegation`.

These reservations do not imply that any other implementation MUST support them. Non-Extol implementations MAY treat them as opaque URIs.

### §2.3 Canonical JSON encoding for the semantic payload

The payload of an attestation (the semantic content being witnessed) is a JSON object whose shape is defined by the activity type's schema. For the payload to be hashable in a deterministic, cross-implementation manner, it MUST be canonicalized before hashing.

Canonicalization procedure follows RFC 8785 (JSON Canonicalization Scheme):

1. Encode all strings as UTF-8 with Unicode NFC normalization.
2. Sort all object keys lexicographically by their UTF-16 code units.
3. Serialize numbers per RFC 8785 §3.2.2.3.
4. Emit no insignificant whitespace.
5. Represent booleans as `true` / `false`, and null as `null`.

Implementations MUST produce identical bytes for identical semantic payloads, regardless of the language, library, or platform used.

### §2.4 activity_hash, data_hash, and source_hash primitives

Three hashes appear in the canonical byte sequence for signing:

- **`activity_hash`** (32 bytes, derived not stored separately): `SHA-256(UTF-8 bytes of the activity_type URI, after Unicode NFC normalization)`. This gives a fixed-length representation of the variable-length activity type URI. Implementations that store the URI need not store this hash; they derive it on demand.

- **`data_hash`** (32 bytes): `SHA-256(canonicalized JSON payload)` per §2.3. This commits to the semantic content of the attestation without requiring the payload to be transmitted, stored publicly, or preserved indefinitely.

- **`source_hash`** (32 bytes): `SHA-256(canonical source identifier)` per §9.2's per-source-type canonicalization rules. This commits to the specific external source the signer relied on.

**Sourceless attestations (normative).** When `source_type ∈ {0, 1}` (that is, `unknown` or `self_reported`), `source_hash` MUST be exactly 32 zero bytes. Signers MUST NOT populate `source_hash` for sourceless attestations. Verifiers MUST reject any attestation whose `source_type` is `unknown` or `self_reported` and whose `source_hash` is not 32 zero bytes; such attestations are malformed at Layer 1 and fail verification independently of signature validity.

Implementations MUST use SHA-256 for all three. Additional hash algorithms may be registered in future versions of this specification.

**Cross-implementation source identity.** For two attestations from different implementations to correctly cross-reference the same source, both implementations MUST use the same canonicalization rule for that source type. The rules are registered in §9.2 alongside the source_type enum values. Where §9.2 specifies a canonicalization procedure as `MUST`, deviation breaks cross-implementation graph analysis. Where §9.2 specifies a procedure as `SHOULD` (typically for source types with no external canonical form, such as `computed` or `system_observed`), implementations MAY adopt implementation-defined canonicalization but SHOULD NOT expect cross-implementation source-identity matches.

### §2.5 Provenance fields (required)

Five fields commit the signer's provenance claim to the canonical byte sequence: `source_hash`, `source_type`, `confidence`, `witnessing_depth`, and `attestor_relationship`. These fields together separate *what the signer relied on* from *how the signer relied on it*, giving verifiers two orthogonal dimensions to reason about the claim's basis.

The full list of registered values for each enum is in §9.2 (source_type), §9.3 (witnessing_depth), and §9.4 (attestor_relationship). Integer positions are stable across versions (§1.4).

**`source_type`** (u16) identifies the kind of source the signer relied on. Fifteen values are registered in v0.1-final, spanning self-reported claims, external authoritative sources (ORCID, DOI, git commits, OpenAlex, IRS/SEC filings, community databases like MusicBrainz), platform observations, computed assertions, and references to other SWORN attestations.

**`source_hash`** (32 bytes) identifies the specific source. It is `SHA-256` of the canonical source identifier per §9.2. When `source_type` is `unknown` or `self_reported`, `source_hash` MUST be 32 zero bytes.

**`confidence`** (u16) is the signer's estimate of the claim's confidence, in basis points from 0 to 10000. A value of 10000 means "as confident as I can be given this source." A value of 0 means "I have no confidence in this claim." Confidence is scalar in v0.1; per-dimension confidence (e.g., separate confidence in subject identity vs. activity participation) is deferred to a future version (see [OPEN_QUESTIONS.md](./OPEN_QUESTIONS.md)).

**Confidence is a signer's snapshot at the moment of signing.** Confidence MUST NOT be revised in place. If a signer later learns their claim was weaker than they originally estimated (for example, an ORCID identifier collision surfaces during dedup), the signer MUST issue a new attestation with the corrected confidence, using the additive-attestation pattern that applies to revocation. Implementations MUST NOT mutate the `confidence` field of a stored attestation.

**`witnessing_depth`** (u8) captures the epistemic depth of the witnessing act itself, orthogonal to source_type. Six values are registered. An ORCID-sourced attestation with `computed_match` depth (a machine matched a name and institution) is a fundamentally different trust artifact than a peer-witnessed attestation with `physically_observed` depth (a person was in the room), even if both are legitimate.

**`attestor_relationship`** (u8) captures the signer's relationship to the subject at the moment of signing. Seven values are registered. This field is a snapshot; a signer's relationship to a subject may change over time (a coordinator steps down, an institution's association ends), but the signed attestation preserves the relationship at signing time.

The `institution` value (attestor_relationship = 6) includes automated ingestion pipelines: when an Extol Cortex worker signs an attestation on behalf of the platform, `attestor_relationship = institution` is appropriate. This clarifies that the field describes the *role* of the signing party, not necessarily a human presence.

#### §2.5.1 Signer's claim, not verifier's guarantee (required)

> Provenance fields express the signer's claim about the source and method. A verifier receiving an attestation with `source_type = orcid` and `confidence = 9500` MUST treat this as the signer's assertion, not the verifier's guarantee. Independent verification of high-confidence sources (public APIs, resolvable DOIs, git commit hashes) is a verifier-side responsibility, not a spec-level requirement.
>
> Implementations MUST NOT reject attestations solely because their provenance is not independently verifiable. Confidence-based reader-side heuristics (weighting, filtering, tier assignment) are implementation-defined. A signer's claim to have relied on ORCID does not become invalid because the verifier cannot reach the ORCID API at query time. Payload URL availability, source URL availability, and external database availability are NOT signature-verification concerns; they are graph-analysis concerns handled at reader-side heuristic layers.

**Rationale.** SWORN's separation of claim from guarantee is what allows the graph to include self-reported claims alongside cryptographically-anchored ones. A low-confidence self-report is still a valid attestation; its weakness is legible via `confidence` and `witnessing_depth`, not by exclusion. Rejecting unverifiable provenance at the signature layer would collapse the whole class of legitimate self-reports the graph is designed to accept.

#### §2.5.2 Dedup semantics (required)

Two attestations that share the same `subject + activity_hash + data_hash` but differ in `signer`, `source_hash`, or `source_type` are independent corroborations. Implementations MUST NOT treat them as duplicates. A verifier walking the graph SHOULD interpret them as multiple parties (or a single party from multiple sources) making the same claim.

Two attestations that share `subject + activity_hash` but have different `data_hash` values are not necessarily contradictions. They are attestations about the same subject and activity type with different payload content. Interpretation is graph-analysis territory.

Reference-attestations (patterns such as `activity_type = correction | dispute | supersession` with `subject` pointing at another attestation's identifier) are addressed at the application layer, not in SWORN core. Implementations that adopt such conventions render the reference graph cleanly; implementations that do not still see the raw edges and can reason about them however they choose.

### §2.6 Subject and witness_for fields

The **`subject`** field is 32 bytes interpreted as one of:

- an Ed25519 public key of another SWORN signer;
- a content hash (32 bytes of hash output, per the activity type's schema);
- an equivalent 32-byte identifier defined by the activity type's schema.

The interpretation is activity-type-defined and MUST be documented in the activity type's schema.

The **`witness_for`** field is always present in the canonical byte sequence (§3.1) and carries one of:

- **32 zero bytes:** the attestation stands alone. The signer is making a first-order claim about the subject.
- **a 32-byte pubkey or content hash:** the attestation endorses or witnesses a specific other party's claim.

`witness_for` enables a signer to say "I saw X's attestation about Y and I corroborate it," without requiring the two-layer role machinery reserved for future versions. Semantics beyond "this signer corroborates that party's claim about this subject" are activity-type-defined.

### §2.7 Timestamp and retention_hint

The **`created_at`** field is a Unix epoch timestamp in seconds (int64), representing when the signer produced the attestation. Signers MUST NOT backdate attestations. Verifiers MAY reject attestations whose `created_at` is more than 300 seconds in the future relative to their local clock, to account for clock skew.

The **`retention_hint`** field is an int64 encoding the signer's intent regarding payload storage duration:

- **`retention_hint > 0`:** Unix epoch seconds after which the payload MAY be discarded by any party storing it. The hash on the ledger remains durable; the payload is not guaranteed to be retrievable after this time.
- **`retention_hint == 0`:** no expiry intent expressed; the implementation-defined default applies.
- **`retention_hint == -1`:** the signer intends the payload to be preserved indefinitely (subject to any implementation's storage limitations).

Retention is a hint, not a guarantee. Verifiers MUST NOT assume payload availability at any future time. The on-chain hash remains verifiable regardless of payload availability (§5.4).

Implementations MAY offer different retention hints as a service tier. If they do, the tiering scheme is subject to §1.5's transparency requirement to the extent that retention differences are exchanged for value.

### §2.8 Source license and off-chain metadata (required)

> Where an attestation's source (as identified by `source_hash` and `source_type`) carries license terms, for example a Creative Commons license on an ORCID record, an open-source license on a git commit, or terms-of-service on a platform-recorded event, those terms bind the attestation and any downstream use of the referenced content. License terms are NOT signed over as part of the canonical byte sequence because they are external, mutable, and updateable by the source.
>
> Implementations MUST preserve license information alongside the attestation record as retrievable metadata, and MUST propagate that information to any verifier they disclose the attestation to. Implementations MUST NOT strip or replace license information when re-emitting attestations.
>
> Signing over the license would have created ambiguity about whether the attestation itself is subject to that license; the split preserves the discipline that a citation carries a reference, not the licensed content.

Implementations SHOULD preserve license information as a triple of `(source_hash, license_identifier, license_effective_range)` rather than as a single mutable field. Source terms change over time (StackOverflow license disputes, GitHub TOS updates, individual OpenAlex-aggregated sources with varying terms); preserving effective-range makes the audit question "was this attestation ingested when the source was still redistributable?" answerable.

Similarly, `quality_flags` (a signer's annotations of claim weaknesses known at attestation time) are NOT signed over, but implementations SHOULD store them alongside the attestation and MUST NOT strip them when re-emitting. Signing over quality flags would have required signers to predict future flag taxonomies; the additive metadata layer allows the flag vocabulary to evolve without breaking signatures.

Implementations SHOULD distinguish provenance produced *at the time of the original attestation* from provenance *backfilled during migration*, via an off-chain `provenance_origin` flag (`original` or `backfilled`). Backfilled provenance is honest data with a different epistemic status than original provenance and should be inspectable as such.

---

## §3 Layer 2: Signing

### §3.1 Canonical byte sequence for signing

Every signature covers the following byte sequence, in this order:

```
canonical_bytes =
      spec_version         (2 bytes, u16 little-endian)   -- per §3.1.1
   || signer               (32 bytes)
   || subject              (32 bytes)
   || activity_hash        (32 bytes)   -- per §2.4
   || data_hash            (32 bytes)   -- per §2.4
   || witness_for          (32 bytes)   -- per §2.6
   || source_hash          (32 bytes)   -- per §2.4, §2.5
   || source_type          (2 bytes, u16 little-endian)   -- per §2.5, §9.2
   || confidence           (2 bytes, u16 little-endian)   -- per §2.5
   || witnessing_depth     (1 byte, u8)                   -- per §2.5, §9.3
   || attestor_relationship (1 byte, u8)                  -- per §2.5, §9.4
   || created_at           (8 bytes, int64 little-endian) -- per §2.7
   || retention_hint       (8 bytes, int64 little-endian) -- per §2.7
   || nonce                (32 bytes)                     -- per §3.4
```

Total length: **248 bytes** (2 + 32 × 6 + 2 × 2 + 1 × 2 + 8 × 2 + 32).

Implementations MUST construct this exact byte sequence and MUST NOT include additional fields, framing, prefix bytes beyond `spec_version`, or other version markers in the signed content. Additional metadata that an implementation wishes to include in its storage or transport layer MUST NOT enter the canonical byte sequence, otherwise cross-implementation verification breaks.

The signature is `signature = Ed25519.sign(signer_privkey, canonical_bytes)` per §3.2, or the equivalent for a registered alternative algorithm (§3.3).

#### §3.1.1 spec_version marker

The `spec_version` field is a u16 (little-endian) at the start of the canonical byte sequence. Registered values:

| Value | Version | Status |
|---|---|---|
| 1 | v0.1-preview | Deprecated. Historical rows only. New attestations MUST NOT use this value. |
| 2 | v0.1-final | Current. All new attestations MUST use this value. |

A verifier receiving an attestation MUST inspect `spec_version` first and dispatch to the correct canonical-byte-sequence layout for that version. Verifiers MUST reject attestations with `spec_version` values they do not recognize as unverifiable (distinct from malformed) and SHOULD report a version-mismatch error rather than a signature-failure error, so implementers can distinguish "reader is behind the registry" from "attestation is corrupt."

Any subsequent breaking change to the canonical byte sequence (new fields, algorithm changes, layout reorderings) advances `spec_version` and reserves a new integer value here. Additive changes that do not alter existing bytes (registration of new source_type values, new activity types) do NOT advance `spec_version`.

#### §3.1.2 Verification procedure

1. Read `spec_version` from the stored attestation. Dispatch to the correct canonical-byte-sequence layout.
2. Reconstruct `canonical_bytes` from the stored attestation per that layout.
3. Verify `signature` against `canonical_bytes` using `signer` as the public key, per RFC 8032 §5.1.7.
4. If step 3 succeeds, the attestation is *authentic*: the holder of the `signer` private key produced this exact byte sequence.

Verification per §3.1 does NOT establish that any given payload matches `data_hash`, nor that any given source matches `source_hash`. To verify the payload, the verifier MUST also compute `SHA-256(canonicalize(payload)) == data_hash` per §2.3–§2.4. To verify the source claim, the verifier MAY consult the external source per §2.5.1 (this is optional and implementation-defined).

### §3.2 Ed25519 (mandatory-to-implement)

Every conforming implementation MUST support Ed25519 signing and verification per RFC 8032, with the following parameter choices:

- Curve: Ed25519 (edwards25519), per RFC 8032 §5.1.
- Key encoding: 32-byte public key, 32-byte private seed, 64-byte signature.
- Message: the 248-byte `canonical_bytes` sequence from §3.1 (for `spec_version = 2`), passed to `Ed25519.sign` without pre-hashing (i.e., PureEdDSA per RFC 8032 §5.1, not Ed25519ph).

Implementations MUST use PureEdDSA. Implementations MUST NOT use Ed25519ph.

### §3.3 Signature algorithm extension mechanism

Future versions of this specification MAY register additional signature algorithms in §9.5. For v0.1, only Ed25519 is defined. Any implementation that encounters an attestation whose algorithm is not Ed25519 MUST reject it as unverifiable. Implementations MUST NOT pass such attestations through as verified.

Because v0.1 defines a single algorithm, no algorithm identifier appears in the canonical byte sequence beyond `spec_version`. Future versions that add algorithms will advance `spec_version` and either introduce an algorithm identifier byte or specify the algorithm implicitly per version. Implementations MUST NOT invent algorithm identifier bytes on their own.

### §3.4 Replay protection (nonce derivation)

The **`nonce`** field is a 32-byte value present in the canonical byte sequence (§3.1) that ensures a signer producing two attestations with otherwise identical fields still produces two distinct signatures over two distinct byte sequences.

The nonce is REQUIRED. The method used to derive it is implementation-defined, subject to the following required properties:

- **Uniqueness.** The nonce MUST be unique per `(signer, subject, activity_hash)` tuple over the practical lifetime of the signing key. Two attestations by the same signer, about the same subject, of the same activity type, MUST NOT share a nonce.
- **Derivation mode.** The nonce MAY be deterministic (e.g., a hash of implementation-defined uniqueness inputs) or random (e.g., 32 cryptographic-random bytes). Determinism is preferable when idempotent re-derivation is desired.
- **Unpredictability.** An attacker who observes a stream of a signer's attestations SHOULD NOT be able to predict the nonce of the next attestation. Nonces MUST NOT be sequential counters visible to third parties.

**Reference derivation (non-normative).** The Extol implementation derives nonces as:

```
nonce = SHA-256(signer || subject || activity_hash || implementation_scope_id)
```

where `implementation_scope_id` is a 32-byte value derived from community and rotation-epoch context. This is presented as an example. Other implementations MAY choose different derivations.

### §3.5 Key rotation considerations (non-normative)

A signer's public key is expected to persist for the lifetime of the standing accrued to it. Rotation is possible but expensive: rotating a key requires either

- (a) publishing a signed statement from the old key delegating to a new key (a pattern reserved for future versions of this specification), or
- (b) accepting that standing accrued to the old key does not transfer.

Implementations that wish to hide the signing key from their users (e.g., using KMS or hardware-backed derivation) MAY do so. The private-key material MAY be re-derivable from a seed rather than stored. The mechanism used to generate keys is out of scope for this specification, provided the resulting signatures verify per §3.1–§3.2.

For v0.1, implementations SHOULD assume signer keys are long-lived. Future versions will define required rotation patterns.

---

## §4 Layer 3: Registry

Layer 3 defines what a signer is, how signer identity persists over time, how signers withdraw prior statements, and how the accumulated graph of attestations is interpreted as standing. It deliberately does not define roles, affiliations, delegation, or any hierarchy above the single-signer type; those are reserved for future versions (§4.5).

### §4.1 Signer identity model (single signer type, v0.1)

A signer is a single Ed25519 public key. There is no registration step, no directory service, no central issuer, and no distinction between signer classes at the protocol layer. Any party that holds an Ed25519 private key can produce a conforming SWORN attestation; the resulting `signer` field in the canonical byte sequence (§3.1) is that private key's corresponding 32-byte public key.

Implementations MUST NOT require signer pre-registration as a condition of accepting an attestation. Rate limiting, quota enforcement, and admission control at the transport layer (§6.5) are separate concerns and MUST NOT be conflated with signer identity.

The identity of a signer is exactly its public key. The mapping between a signer and any real-world person, organization, or automated system is implementation-defined and lives outside the protocol. Implementations that wish to associate signers with human-readable labels, verified identities, or organizational affiliations MAY do so in an off-chain metadata layer, subject to the transparency requirement of §1.5 where such labels are used to convert standing into value.

### §4.2 Persistent key semantics

Signer identity persists exactly as long as the signer's private key persists. Two attestations bearing the same `signer` value are, at the protocol layer, produced by the same signer regardless of when they were signed or by what implementation. This is the primitive that makes standing accumulate across time and across implementations.

Implementations MUST NOT treat the same `signer` public key as two distinct signers under any circumstance. Implementations MAY offer signers a mechanism to label themselves (display names, avatars, affiliations) but MUST NOT let such labels override the cryptographic identity of the key.

Signers MAY hold multiple distinct keys and use them in different contexts (per-community keys, per-purpose keys, disposable keys). Each key is a distinct signer at the protocol layer. Implementations MAY offer off-chain grouping ("these keys belong to the same real-world entity") but the protocol makes no such grouping.

For v0.1, implementations SHOULD assume signer keys are long-lived. Formal key rotation semantics are reserved for future versions (§3.5). A signer that rotates keys today does so by starting a fresh signer identity; prior attestations under the old key remain valid but no longer accumulate standing to the new key.

### §4.3 Revocation by additive attestation

SWORN has no on-chain mutation of prior attestations. Revocation is expressed by signing a new attestation that references the target.

**Required convention.** An attestation whose `activity_type` URI resolves to a revocation-class schema (for example, `sworn.dev/v1/revocation`) and whose `subject` field is `SHA-256(target_canonical_bytes)` constitutes a revocation of the referenced target attestation by the signing party. The subject encoding is the same primitive used by `source_type = 14` (`external_sworn_attestation`) in §9.2, giving the whole additive-reference family (revocations, corrections, disputes, supersessions, endorsements of attestations) a single convention.

Signers MUST populate `subject` with the exact 32-byte SHA-256 of the target's canonical byte sequence (§3.1). Implementations MUST NOT substitute other identifiers (attestation UUIDs, substrate-specific PDA addresses, human-readable labels) for `subject` in a revocation; those may live off-chain as metadata but do not participate in the cryptographic reference.

Implementations MUST NOT delete, mutate, or otherwise alter the target attestation's record in response to a revocation. Both records remain durable. Verifiers walking the graph MUST see both the original attestation and any revocations of it, and MAY apply reader-side policy about how to weigh them.

**Who can revoke.** Only the original signer of an attestation can revoke it in a way that discounts its standing at the reader's discretion. A revocation signed by a party other than the original signer is a first-class attestation in its own right (a "dispute" or "counter-claim") but is not a revocation of the target attestation's cryptographic validity. The original signature over the target's canonical bytes remains cryptographically valid regardless.

**Dangling revocations.** A verifier MUST verify the revocation's signature per §3.1's verification procedure. A verifier MAY consider a revocation "applied" only to attestations whose canonical bytes hash to the value in `subject`. Revocations whose `subject` does not match any known attestation are legitimate signed statements but have no target to apply to; verifiers SHOULD retain them (a matching attestation may surface later, having previously been unknown to that verifier) but MUST NOT apply them to non-matching attestations even when the signer of the revocation matches the signer of some other attestation.

**Additive-only rule.** Implementations MUST treat every revocation as an additive record in the notarization substrate. Implementations MUST NOT provide a "hard delete" or "unpublish" path at Layer 4 (§5.4). This is what makes SWORN's revocation semantics honest: a revocation is a fact about the signer's later intent, not an erasure of prior fact.

**Interaction with key rotation.** Because v0.1 has no formal key rotation (§3.5), revocation requires the original signing key. If a signer has lost access to the key that produced an attestation, that attestation cannot be revoked in the v0.1 sense. The signer MAY sign a new attestation under a new key stating "I have lost control of key X and no longer stand behind attestations signed by it"; such a statement is a first-class attestation but does not carry the same weight as a same-key revocation. Formal key-compromise semantics are reserved for future versions.

**Query shape.** Because `subject = SHA-256(target_canonical_bytes)` is substrate-neutral and self-computing, the query "find revocations of attestation X" is `WHERE activity_type = revocation-class AND subject = SHA-256(X_canonical_bytes) AND signer = X.signer` in every implementation, without per-substrate identifier translation. Implementations MAY maintain an index on `subject` restricted to revocation-class rows for efficiency.

### §4.4 Standing as an emergent property of the graph

Standing is the accumulated record of attestations signed by, and attestations naming, a given signer. Standing is not a score, a ranking, or a value. Standing is a graph.

**Required semantics.** Implementations MUST expose sufficient primitives at Layer 5 (§6) for a verifier to walk the graph and reason about a signer's attestation history. Implementations MUST NOT present derived signals (rankings, tier assignments, aggregate scores) as attestations. The distinction between the attestation graph (facts anyone can independently verify) and derived signals (interpretations one implementation has chosen to compute) is required by §1.5.

**Implementation freedom.** Given the same underlying graph, two implementations MAY compute different derived signals for legitimate reasons: different thresholds for weighting peer-witnessed vs computed-match attestations (§9.3), different treatment of low-`confidence` self-reports (§2.5.1), different decay curves for older attestations. This is expected and healthy. What implementations MUST NOT do is present these interpretations as if they were themselves signed attestations.

**Cross-implementation portability.** Because Layer 1 (§2) fully specifies the canonical byte sequence and Layer 2 (§3) fully specifies signature semantics, any two implementations that produce or consume attestations do so against the same graph. A signer's standing, measured as raw attestation counts and relationships, is identical across implementations. Derived signals differ; the graph does not.

**Emergent revocation.** Standing loss from revocation is not a spec-level mechanism. A revocation attestation (§4.3) is a fact in the graph; how much it decreases the referenced attestation's contribution to standing is a reader-side policy decision. Implementations SHOULD document their revocation-weighting policy per §1.5 where standing is converted into value.

### §4.5 What is NOT in this layer (roles, affiliation, delegation; reserved for future)

The following patterns exist in real recognition systems, are named here for completeness, and are explicitly out of scope for v0.1:

- **Two-layer witness/certifier roles.** A pattern where an entity (organization) endorses a persona (individual signer) and can sign entity-binding statements distinct from persona-signed statements. Requires role primitives Layer 3 v0.1 deliberately omits. Reserved for a future companion specification.
- **Affiliation.** A signed relationship of membership between a persona and an entity, with revocation semantics distinct from attestation revocation. Requires the roles model above.
- **Delegated attestation.** A pattern where signer A grants signer B the authority to sign on A's behalf, with revocable grant semantics. Materially expands the trust model and requires its own spec pass. Reserved for a future version.
- **Multi-signature attestations.** An attestation requiring N-of-M signatures before it becomes valid. Distinct from independent attestations by multiple signers about the same subject (which v0.1 already supports natively via §2.5.2). Reserved for a future version.
- **Directory or discovery services.** No spec-level mechanism for looking up a signer by name, resolving a signer to a real-world identity, or discovering signers by attribute. Implementations MAY offer such services off-chain; the protocol makes no commitment.

Implementations MAY build any of these on top of v0.1's single-signer-type primitive, as application conventions in their own right. Where they do, those conventions live in implementation documentation, not in this specification.

---

## §5 Layer 4: Notarization

Layer 4 defines how an attestation's cryptographic identity is committed to a public, tamper-evident ledger so that any verifier, at any later time, can confirm the attestation existed at a specific point in time. The layer is substrate-agnostic: SWORN commits only to what must be published and how it must behave, not to which system publishes it. The two live substrate bindings maintained alongside this specification (Postgres via [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres) and Solana SAS via Extol's production implementation) are documented in Appendix A and Appendix B.

### §5.1 Hash-anchor commitment (required)

The notarization substrate MUST publish, per attestation, at least the following:

- **The `data_hash`** of the attestation as defined in §2.4 (32 bytes, SHA-256).
- **The `signer`** public key (32 bytes) OR a substrate-native identifier from which the signer public key is deterministically resolvable.
- **A substrate-native creation timestamp** (block time, commit time, log sequence number, or equivalent) that is monotonic within the substrate and that any verifier can independently read.

The full 32-byte SHA-256 of the attestation's canonical byte sequence per §3.1 (denoted `attestation_hash` in the remainder of this section) is the substrate-agnostic identifier for cross-referencing an attestation from any other attestation (§2.5.2, §4.3, §9.2 source_type=14). Substrates MAY store additional attestation content on-chain (larger fields, entire canonical bytes, signature) at the implementation's discretion; substrates MUST store at least `attestation_hash` in a form the verifier can compute from the substrate's record without out-of-band information.

**Anchor-produces, does-not-attest.** A notarization substrate publishing an attestation's hash is NOT itself attesting to the content of the attestation. The substrate's role is temporal: it establishes that the hash existed at a known point in time. Interpretations of the attestation live at Layer 3 (§4.4) and are reader-side concerns.

**Independent hash recomputation.** For a verifier to trust the on-chain hash, the verifier MUST be able to independently reconstruct the canonical byte sequence (§3.1) from the attestation's stored fields and compute `SHA-256(canonical_bytes)`, then confirm the result matches what the substrate published. A substrate implementation that publishes a hash the verifier cannot independently reproduce from stored source material is NOT conforming. This property is what makes the three-implementation round-trip demo (Appendix C) meaningful: each party recomputes rather than trusting upstream.

**Anchoring is not signing.** The party operating the notarization substrate MAY be a different party than the signer. Any party that possesses the canonical bytes and the signer's signature can compute the attestation's hash and publish it to the substrate; the signature is not touched or replaced during anchoring. This separation is required for the "sign locally, anchor via a service" pattern common in resource-constrained clients.

### §5.2 Merkle batching (deferred)

Batching multiple attestation hashes into a Merkle tree and anchoring only the root is a well-understood cost-reduction pattern used by several existing substrates. SWORN v0.1 deliberately does NOT specify a Merkle batching format at the protocol layer, for two reasons:

1. **Implementation experience.** Extol's production Solana SAS binding uses Merkle batching as a per-tier cost optimization; sworn-postgres does not. Cross-implementation verification of batched proofs requires more design work than is warranted by two implementations, one of which does not currently need the feature.
2. **Format lock-in risk.** Once a Merkle format is normative, changing it becomes a v1.0 breaking change (§3.3 analogy). Waiting until at least two independent implementations need it produces better format than committing early.

**Interim guidance.** Implementations that batch (such as the Solana SAS binding documented in Appendix A) MUST anchor the resulting Merkle root as a distinct attestation-like record at Layer 4, and MUST provide off-chain inclusion proofs to verifiers on request. Inclusion proofs verified against a batched root MUST resolve to the individual attestation's `data_hash` per §5.1's independent-recomputation rule. Implementations SHOULD document their batching format in implementation notes; a normative Merkle format is reserved for v0.2.

**Cross-substrate Merkle interoperability** (a verifier holding an inclusion proof from substrate A confirming an attestation is anchored to substrate B) is out of scope for v0.1.

### §5.3 Retention semantics (per-record, differing retention allowed)

Layer 1's `retention_hint` field (§2.7) expresses the signer's intent regarding payload retention. Layer 4 governs what happens to the on-chain hash under the same hint.

**Required rule.** The on-chain hash MUST remain published for at least as long as the substrate maintains any record of the attestation. Substrates MUST NOT selectively expire on-chain hashes ahead of the substrate's own record-retention policy. If a substrate deletes the record entirely (subject to substrate policy), the hash goes with it; there is no on-chain-hash-without-substrate-record state.

**Per-attestation heterogeneity.** A single notarization substrate MAY carry attestations with different `retention_hint` values, and those attestations MAY have different off-chain payload availability at any given time. A verifier MUST NOT assume all attestations in the same substrate share the same retention posture.

**Retention is not validity.** An attestation whose `retention_hint` has passed and whose payload has been reclaimed is still a valid attestation at Layer 2. Its signature verifies. What has changed is its *disclosability* (§6): the payload may no longer be retrievable, so a verifier may see only metadata. The distinction between "invalid" and "undisclosable-but-valid" is preserved by never mutating the on-chain hash.

### §5.4 Durability guarantees for the on-chain hash (required)

The on-chain hash of an attestation, once published, MUST NOT be revoked, mutated, or reversed by the notarization substrate under any protocol-visible circumstance. This is the specification-level durability guarantee: a verifier reading the substrate can rely on the invariant that a hash present today was present at its published timestamp and will remain present for as long as the substrate itself exists.

**Substrate-level failure modes** (chain reorganizations, ledger rollbacks, database restores) are out of protocol scope. Implementations MUST document what substrate-level guarantees they provide; verifiers MAY treat attestations from substrates with weaker durability differently in reader-side heuristics. But nothing at the protocol layer permits a substrate to selectively withdraw an attestation's hash while preserving others.

**Revocation is additive, not durable-hash-mutating.** Per §4.3, revocation is expressed as a new attestation whose `subject` is `SHA-256(target_canonical_bytes)`. The revocation attestation itself is anchored per §5.1; the target attestation's on-chain hash is unaffected. A verifier walking the substrate sees both hashes and applies reader-side policy about how to weigh the revocation against the target (§4.4). Substrates MUST NOT interpret a revocation as license to delete or hide the target's on-chain hash. This is the anti-erasure discipline: an attestation once published, and any later revocations of it, are both matters of durable public record.

**Substrate compromise.** If a substrate is discovered to have violated its published durability guarantees (an operator forcibly deleted hashes, a chain executed a policy-driven rollback), attestations previously anchored to it lose the temporal grounding the substrate was providing. Signatures over those attestations remain cryptographically valid, but the "the hash existed at time T" property collapses. Implementations SHOULD document a recovery posture: whether to re-anchor affected attestations to a different substrate, whether to accept substrate-A attestations as substrate-B attestations for graph-analysis purposes, and whether reader-side heuristics should discount attestations anchored to compromised substrates. Substrate-compromise recovery is not standardized in v0.1.

### §5.5 Off-chain payload storage (implementation-defined)

Payloads are NOT stored on-chain by requirement of this specification. The on-chain record (§5.1) commits to the `data_hash`; the payload itself lives off-chain, typically in the implementation's application database (as in sworn-postgres and Cortex) or in an external content-addressed store.

**Payload retrieval is not a Layer 4 concern.** The presentation layer (§6) governs how a verifier obtains a payload, subject to the consent semantics of §6.2. Layer 4's role ends at the hash commitment; making the payload retrievable is a Layer 5 responsibility.

**Reclaimed payloads and hash validity.** A payload reclaimed per §2.7's `retention_hint` semantics does NOT affect the on-chain hash's validity. Verifiers that receive only metadata (the on-chain hash and the attestation's field structure without a retrievable payload) can still confirm the signature is valid, can still verify the signer identity, and can still walk the graph to see revocations and corroborations. What they cannot do is verify that any candidate payload they encounter later matches the `data_hash`; that check requires the payload itself.

**Substrate MAY store payloads.** Nothing in this specification prevents a substrate from storing the full payload on-chain if the substrate's storage economics allow it. Substrates that do so simply collapse §5's off-chain-payload assumption into on-chain storage. The verifier's independent-recomputation rule (§5.1) still applies: the payload's canonicalized-JSON SHA-256 MUST equal the published `data_hash`.

### §5.6 What is NOT in this layer (specific chain choice, PDA layouts; see appendices)

The following are explicitly out of scope for §5's required text:

- **Choice of substrate.** Whether an implementation uses Solana, Ethereum, Bitcoin, a Postgres append-only table, a git-anchored log, or a certificate-transparency log is an implementation decision. SWORN v0.1 requires only that the substrate satisfy §5.1's publication rule and §5.4's durability rule.
- **Substrate-specific data formats.** How a Solana PDA is structured, how a Postgres row is laid out, how an Ethereum event is encoded: all substrate-defined. Appendix A documents the Solana SAS binding; Appendix B documents the Postgres binding. Neither is normative for other substrates.
- **Substrate-native identifiers as attestation subjects.** An attestation MAY reference a substrate-native identifier (a Solana account address, an Ethereum transaction hash, a git commit) as its `subject` when the activity type's schema calls for it, but such references are activity-type-defined and MUST NOT substitute for the SHA-256 hash primitive when the target is another SWORN attestation (§4.3).
- **Cost, throughput, and operational economics.** Which substrate is cheapest per attestation, which is fastest to publish, which offers the strongest durability guarantees: all substrate-choice concerns. SWORN v0.1 makes no recommendation. Implementations SHOULD document their substrate's cost and throughput characteristics so integrators can choose accordingly.
- **Substrate discovery.** How a verifier learns which substrate anchors a given attestation is a Layer 5 concern (§6.1's verification-endpoint contract) or an application-layer concern, not a Layer 4 mechanism.

Implementations that wish to add substrate-specific behavior (batched anchoring, on-chain payload storage, custom retention policies) MAY do so provided the required text of §5.1 through §5.4 is preserved. Where an implementation's substrate cannot satisfy the required text (for example, a substrate that does not guarantee hash durability), that substrate is NOT conforming for SWORN attestations regardless of what else it provides.

---

## §6 Layer 5: Presentation

Layer 5 defines how third parties interact with attestations after Layer 4 has anchored them. Its central discipline is *shown, not pulled*: a conforming implementation MUST enable verification of attestations a caller already knows about, and MUST NOT enable enumeration, discovery, or bulk export of attestations by properties of their subjects or signers. This is the property that keeps the SWORN graph from becoming a surveillance database while preserving its utility as verifiable testimony.

### §6.1 Verification endpoint contract

A conforming implementation MUST expose a verification interface that, given an attestation identifier known to the caller, returns:

- the full canonical bytes of the attestation (or the primitive fields sufficient for the caller to reconstruct them per §3.1),
- the signature (64 bytes, per §3.2),
- the `activity_type` URI (from which `activity_hash` is derived per §2.4),
- the substrate-published `attestation_hash` (per §5.1) sufficient for the caller to confirm anchoring against the notarization substrate,
- and the metadata required to interpret provenance fields (per §2.5).

The verification interface MUST NOT return the payload as part of this call. Payload disclosure is a distinct operation per §6.2.

Implementations MAY offer the verification interface over any transport (HTTP, gRPC, CLI, a substrate query directly, an in-process library call). SWORN v0.1 does not mandate a specific transport, but the reference implementation exposes it as HTTP+JSON at `GET /attestations/{id}` for metadata and `GET /verify/{id}` for a server-computed valid/invalid answer (see Appendix B and [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres)'s `openapi.yaml`).

**Independent verification requirement.** Regardless of transport, a caller receiving the verification-endpoint response MUST be able to independently reconstruct the canonical bytes, recompute `SHA-256(canonical_bytes)` and confirm it matches the substrate-published `attestation_hash`, and verify the Ed25519 signature. An implementation whose response cannot be independently verified in this way is NOT conforming, even if it returns a valid-looking status. The trust primitive is the caller's own recomputation, not the endpoint's assertion.

### §6.2 Two-call design (verify metadata, disclose payload with subject consent)

Verification and payload disclosure are separate operations. Verification (§6.1) reveals who signed what class of claim about which subject, plus the hash commitment to the payload's content. Disclosure reveals the payload itself.

**Required separation.** Implementations MUST NOT return payload content from the verification interface. Implementations MUST require an explicit disclosure authorization (per §6.3) before returning payload bytes.

**Rationale.** A verifier who wants to know "did signer X make a claim of type Y about subject Z at time T" can answer that question from the verification interface alone, without seeing the specific content of the claim. Many use cases (aggregate counts, standing-graph traversal, temporal analysis, dispute resolution about who signed what) require only the metadata. Payload content is often more sensitive than the fact of the attestation and should require a separate, signer-authorized step to retrieve.

**Payload authenticity check on disclosure.** When an implementation returns a payload via a disclosure call, the caller MUST be able to compute `SHA-256(canonicalize(payload))` and confirm equality with the `data_hash` in the verified canonical bytes. A disclosed payload whose recomputed hash does not match `data_hash` MUST be rejected by the caller as tampered or misdelivered, even if the disclosure token was valid. The signature over `data_hash` is the payload's authenticity guarantee; the disclosure channel is a delivery mechanism, not a trust primitive.

### §6.3 Disclosure token semantics

A disclosure token authorizes exactly one retrieval of one attestation's payload. The token binds three properties: the specific attestation (by ID or hash), a time window during which the token is redeemable, and a single-use consumption guarantee.

**Required properties.**

- **Single-use by default.** A conforming implementation MUST redeem each disclosure token at most once. A second redemption attempt MUST fail (the reference implementation returns HTTP 410 Gone with an explanatory body). Implementations MAY offer explicitly-designated multi-use tokens as a separate token type; where they do, the multi-use property MUST be visible in the token's metadata so a caller can distinguish.
- **Time-bounded.** Every token has an expiration. Implementations MUST reject expired tokens with a distinct error class from single-use exhaustion (so callers can distinguish "you already used this" from "this window has closed"). SWORN v0.1 recommends a minimum floor of 60 seconds and a maximum ceiling of 7 days for single-use tokens; implementations MAY tune within that range.
- **Signer-authorized.** A disclosure token MUST be issued by proof of control of the attestation's signing key or by a mechanism the signer has explicitly authorized. Implementations MUST NOT permit unauthenticated parties to mint disclosure tokens for arbitrary attestations. The reference implementation binds token issuance to an Ed25519 signature by the attestation's signer over a domain-separated issuance message; other equivalent mechanisms (session-authenticated UI approval by the signer, delegated authorization within a signer-controlled application, cryptographic capability tokens) are acceptable provided the signer's authorization is present at issuance time.
- **Domain-separated.** The bytes signed to authorize token issuance MUST NOT be substitutable for the canonical byte sequence of any attestation (§3.1) or the canonical form of any other SWORN operation. The reference implementation prefixes issuance bytes with the literal string `sworn-disclosure-token-v1`; other implementations MUST use an equivalent domain separator that cannot collide with attestation canonical bytes.

**Non-normative guidance on UX.** The disclosure token is protocol plumbing. How it reaches the requester (URL, QR code, in-app deep link, approval prompt, capability delegation) is an application-layer decision. The single-use default should not be interpreted as requiring the token itself to be human-visible; well-designed implementations often hide the token entirely behind a "requester asks, signer approves in-app, payload delivered" flow.

### §6.4 Refused operations (list-by-subject, bulk export, name search)

A conforming implementation MUST NOT expose operations that enumerate attestations by properties of their subjects, signers, or payloads. Specifically, the following operations MUST be refused (returned as errors, not silently unsupported):

- **List by subject.** A query returning all attestations naming a given subject.
- **List by signer.** A query returning all attestations produced by a given signer.
- **Bulk export.** A query returning attestations without a specific caller-supplied identifier.
- **Name or attribute search.** A query returning attestations whose payload content matches a search pattern.
- **Signer discovery.** A query returning signers matching a real-world identity, label, or attribute.

Implementations MUST return an explicit refusal (the reference implementation uses HTTP 400 with an error body identifying the operation as refused by design) rather than treating these as absent features. The refusal is a first-class part of the presentation contract: an implementer testing conformance MUST observe the refusal to confirm the discipline is enforced.

**Rationale.** SWORN's design commitment is that the attestation graph is *verifiable* without being *enumerable*. A verifier holding an attestation identifier (however obtained) can confirm its authenticity; a party who does not hold an identifier cannot bulk-discover the graph's contents. This is what prevents SWORN from becoming a portable dossier system. The discipline is enforced at Layer 5 because it is the layer where callers meet the system; a substrate that stores attestations without a Layer 5 wrapper (a raw Solana PDA scan, for example) has bypassed the discipline, and any implementation exposing such a raw view to callers is NOT conforming as a SWORN Layer 5 presentation surface.

**Signer-scoped exceptions.** A signer authenticated to their own key MAY retrieve a list of their own attestations from an implementation that stores them; this is a self-service reflection, not enumeration by third parties. Implementations offering this MUST authenticate the request as coming from the signer's key (via signature, session tied to the key, or equivalent).

**Application-scoped exceptions.** An implementation MAY expose enumeration within a bounded application context (for example, all attestations attached to a specific event, group, or resource the caller has independent access to) provided the enumeration is scoped by an application-layer resource identifier the caller must possess. Enumeration by properties of the attestation's signer or subject remains refused.

### §6.5 Rate limiting and abuse considerations

Implementations MUST offer rate limiting on the verification and disclosure interfaces. Rate limiting is not part of the trust model (§4.1 is explicit that admission control is separate from signer identity), but is required at Layer 5 to prevent enumeration-by-timing (where an attacker probes many candidate identifiers to discover which resolve) and to prevent disclosure-endpoint abuse (where an attacker attempts to burn valid tokens with speculative redemptions).

**Required posture.**

- The verification interface (§6.1) SHOULD carry a per-caller rate limit sufficient to prevent enumeration probing. Precise numeric thresholds are implementation-defined; the reference implementation uses 60 requests per minute per source IP as a starting point.
- The disclosure interface (§6.2, §6.3) MUST carry a stricter rate limit than the verification interface. Disclosure returns payload bytes, and lax rate limiting on disclosure functionally reintroduces bulk export. The reference implementation caps disclosure at 6 requests per minute per source IP.
- Implementations MUST distinguish rate-limited responses from other error classes (the reference implementation uses HTTP 429 with a distinct body) so callers can back off appropriately.
- Rate-limit tracking MUST NOT create a signer profile. Implementations MAY track per-IP or per-session request counts but MUST NOT correlate rate-limit state with signer identity in a way that reintroduces signer-linked enumeration through the back door.

**Non-normative guidance.** Production deployments will typically front SWORN implementations with a dedicated reverse proxy, CDN, or API gateway that carries the actual rate-limiting load; the reference implementation's in-process rate limiter is a demonstration of the pattern, not a production-scale solution. Implementations SHOULD document their expected deployment posture so integrators can configure their edge appropriately.

---

## §7 Security considerations

### §7.1 Sybil resistance (bounded, not absolute)
### §7.2 Attack cost model
### §7.3 Colluding attestation rings, graph-analysis detection
### §7.4 Key compromise and revocation
### §7.5 Payload availability vs. hash durability

---

## §8 Privacy considerations

### §8.1 Public verification, private payloads
### §8.2 Subject-mediated disclosure
### §8.3 Right to be forgotten and immutable hashes
### §8.4 Pseudonymity of witnesses

---

## §9 IANA-style registries (or equivalent for a young spec)

### §9.1 Activity type namespace registry

This registry lists namespaces reserved for use with SWORN activity type URIs. Registration in v0.1 is descriptive, not prescriptive: an implementation is free to use any well-formed URI as an activity type (§2.2), and this section documents namespaces already in use so implementers can align without stepping on each other. A formal registration process may land in a future version.

**Reserved namespaces (informative):**

| Namespace prefix | Owner / source | Purpose |
|---|---|---|
| `work.extol.attestation/v1/` | Extol, Inc. | The first production adopter's original vocabulary (see §2.2). |
| `sworn.dev/v1/` | The SWORN specification | Reserved for well-known types defined by future spec revisions (e.g., a `revocation` type in v0.2). |
| `credit.niso.org/contributor-roles/` | NISO (Z39.104-2022) | CRediT (Contributor Roles Taxonomy). See §9.1.1. |

Non-reserved namespaces (any URI a signer chooses to use) remain valid activity types; the registry exists so widely-shared vocabularies do not collide.

#### §9.1.1 CRediT (Contributor Roles Taxonomy)

CRediT (Contributor Roles Taxonomy) is a fourteen-role vocabulary developed by CASRAI and now maintained by NISO as ANSI/NISO Z39.104-2022. Its purpose is to provide transparency in contributions to scholarly published work, enabling systems of attribution, credit, and accountability that go beyond traditional authorship.

SWORN registers the CRediT namespace so that attestations recognizing research contributions can share a single, widely-adopted vocabulary. Implementations SHOULD prefer CRediT roles over ad-hoc alternatives when attesting to research contributions.

**URI pattern.** Each CRediT role maps to a URI under `credit.niso.org/contributor-roles/<slug>`, where `<slug>` is a lowercase kebab-case rendering of the role name.

**Fourteen roles:**

| Slug | Role | Definition (summary) |
|---|---|---|
| `conceptualization` | Conceptualization | Ideas; formulation of overarching research goals and aims. |
| `data-curation` | Data curation | Management activities to annotate, scrub, and maintain research data. |
| `formal-analysis` | Formal analysis | Statistical, mathematical, or computational analysis of study data. |
| `funding-acquisition` | Funding acquisition | Acquisition of financial support for the project. |
| `investigation` | Investigation | Conducting the research process; performing experiments or evidence collection. |
| `methodology` | Methodology | Development or design of methodology; creation of models. |
| `project-administration` | Project administration | Management and coordination of the research activity. |
| `resources` | Resources | Provision of study materials, reagents, patients, samples, compute resources, etc. |
| `software` | Software | Programming, software development, algorithm design, implementation, testing. |
| `supervision` | Supervision | Oversight and leadership responsibility for the research activity. |
| `validation` | Validation | Verification of the overall reproducibility of results and other experimental outputs. |
| `visualization` | Visualization | Preparation, creation, and presentation of published work, specifically visualization. |
| `writing-original-draft` | Writing, original draft | Preparation, creation, and presentation of published work, specifically writing the initial draft. |
| `writing-review-editing` | Writing, review & editing | Critical review, commentary, or revision of published work. |

**Degree of contribution.** CRediT allows an optional degree qualifier of `lead`, `equal`, or `supporting` when multiple contributors share the same role. Implementations expressing this SHOULD carry the degree in the attestation payload (not in the activity type URI). Example payload field: `"contribution_degree": "lead"`.

**Attribution note.** SWORN reserves the `credit.niso.org/contributor-roles/` namespace for interoperability. The CRediT taxonomy itself is owned and maintained by NISO. Implementations using these URIs are consuming CRediT, not extending it.

### §9.2 source_type registry

The `source_type` field (§2.5) is a u16 whose registered values are given below. Integer positions are stable per §1.4. Future additions append at the next unused integer.

For each value, this registry specifies:

- **Slug:** the canonical string label.
- **Description:** what the source represents.
- **source_hash canonicalization:** how implementations MUST derive `source_hash` for this source_type. Cross-implementation source-identity matching depends on all implementations agreeing on this procedure. Where the procedure is marked `SHOULD`, cross-implementation matching is a best-effort convention.

| # | Slug | Description | source_hash canonicalization |
|---|---|---|---|
| 0 | `unknown` | Source not classified. | `source_hash` MUST be 32 zero bytes. |
| 1 | `self_reported` | The signer is asserting a fact about themselves with no external source. | `source_hash` MUST be 32 zero bytes. |
| 2 | `orcid` | Sourced from an ORCID record. | MUST: `SHA-256` of the 19-character ORCID identifier as ASCII, upper-hyphen form (e.g., `0000-0002-1825-0097`). No scheme, no host, no trailing whitespace. |
| 3 | `doi` | Sourced from a DOI-resolvable publication. | MUST: `SHA-256` of the bare DOI in lowercase ASCII (e.g., `10.1234/example.5678`). No `doi.org/`, no scheme, no fragment. |
| 4 | `openalex` | Sourced from an OpenAlex record. | MUST: `SHA-256` of the uppercase OpenAlex ID with type prefix (e.g., `W1234567890`, `A1234567890`). No scheme, no host. |
| 5 | `git_commit` | Sourced from a specific git commit. | MUST: `SHA-256` of the full 40-character lowercase hexadecimal commit SHA. No repo prefix, no branch context. |
| 6 | `rss_parsed` | Machine-extracted from an RSS/Atom feed. | SHOULD: `SHA-256` of the item's `<guid>` or `<atom:id>` value as UTF-8. Where the feed provides neither, SHOULD use the item's canonical URL after Unicode NFC normalization. |
| 7 | `open_source_project` | Sourced from a project's declared authorship (CITATION.cff, package metadata, etc.). | SHOULD: `SHA-256` of the primary repository URL (lowercase scheme+host+path). |
| 8 | `coordinator_confirmed` | A coordinator role in the community affirmed the claim. | SHOULD: `SHA-256(coordinator_signer_pubkey || confirmation_timestamp_int64_le)`. Implementation-specific but MUST be deterministic within an implementation. |
| 9 | `peer_witnessed` | A peer directly witnessed the claimed activity. | MUST: `SHA-256` of the peer's 32-byte SWORN signer pubkey. |
| 10 | `computed` | The claim was derived algorithmically from other data. | SHOULD: `SHA-256(algorithm_identifier_utf8 || canonicalized_inputs_bytes)`. Implementations SHOULD choose canonicalization such that identical computations produce identical `source_hash` values. |
| 11 | `system_observed` | The claim is a system-observed fact (attendance record, transaction confirmation, computed platform stat). | SHOULD: `SHA-256(platform_identifier_utf8 || event_id_utf8)`. Implementation-specific. |
| 12 | `regulatory_filing` | Sourced from a legally-mandated public disclosure (IRS 990, SEC filing, court record, campaign finance filing). | MUST: `SHA-256(filing_type_identifier_utf8 || filing_identifier_utf8)`. E.g., `990:EIN:12345:2024` for an IRS 990 filing, `SEC:accession_number` for an SEC filing. |
| 13 | `community_curated_db` | Sourced from a community-edited database with revision history (MusicBrainz, Wikidata, OpenStreetMap, Discogs). | MUST: `SHA-256` of the canonical entity URL after Unicode NFC normalization (e.g., `https://musicbrainz.org/artist/<mbid>`, `https://www.wikidata.org/wiki/Q<id>`). |
| 14 | `external_sworn_attestation` | References another SWORN attestation (federation, cross-implementation graph). | MUST: `SHA-256` of the referenced attestation's canonical byte sequence, as computed per §3.1 of the version that attestation was signed under. Equivalent to the referenced attestation's stable identifier. |

**Enum evolution.** Additions to this registry (new integer positions) are additive and do NOT advance `spec_version`. Verifiers that encounter a `source_type` value they do not recognize MUST report a version-mismatch condition (distinct from malformed-attestation) so upstream tooling can distinguish "reader is behind" from "attestation is corrupt." Verifiers MUST NOT interpret unknown source_type values as `unknown` (0); the enum is exhaustive at each version.

### §9.3 witnessing_depth registry

The `witnessing_depth` field (§2.5) is a u8. Integer positions are stable per §1.4.

| # | Slug | Meaning |
|---|---|---|
| 0 | `unspecified` | Depth not classified. Legitimate for backfilled records or where the signer chooses not to characterize. |
| 1 | `physically_observed` | The signer was present when the claimed activity occurred and directly witnessed it. |
| 2 | `reviewed_artifacts` | The signer inspected outputs (code, papers, receipts, records) after the fact and forms the claim from that inspection. |
| 3 | `ui_confirmed` | The signer confirmed the claim through a user-interface action (e.g., pressed a button in a client application) without deeper inspection. |
| 4 | `computed_match` | The signer is a machine process that produced the claim by matching against reference data. |
| 5 | `self_asserted` | The subject and signer are the same party; no separate witnessing occurred. |

### §9.4 attestor_relationship registry

The `attestor_relationship` field (§2.5) is a u8. Integer positions are stable per §1.4.

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

Ed25519 is the sole registered signature algorithm in v0.1. Future versions may register additional algorithms per §3.3.

### §9.6 Notarization substrate registry

Substrate choice is implementation-defined; interoperability of hash-anchor commitments across substrates is achieved by the substrate identifier appearing in implementation binding documentation (Appendix A, B). Future versions may formalize this registry.

---

## §10 Conformance

### §10.1 Conformance levels
### §10.2 Interoperability tests
### §10.3 Registration process (or self-declaration during RFC period)

### §10.4 Reference test vectors

Cross-implementation verification is anchored by golden test vectors published alongside this specification at `fixtures/attestations/v0.1-final/` in the [extol-work/sworn](https://github.com/extol-work/sworn) repository. Each vector specifies:

- `input_fields`: the full set of attestation fields per §2.1.
- `expected_canonical_bytes_hex`: the 248-byte canonical byte sequence per §3.1, in hexadecimal.
- `expected_signature_hex`: the 64-byte Ed25519 signature over the canonical bytes, in hexadecimal.
- `notes`: what the vector exercises (edge cases, provenance shape, etc.).

A conforming implementation MUST reproduce `expected_canonical_bytes_hex` and `expected_signature_hex` byte-for-byte from `input_fields` for every published vector. Any deviation indicates a serialization, canonicalization, or signing bug that would break cross-implementation verification.

Implementations SHOULD contribute additional vectors as they discover edge cases in production use.

---

## Appendix A: Solana / SAS binding (informative)

Extol's on-chain implementation notes. First production adopter. Not required.

## Appendix B: Postgres binding (informative)

Reference implementation notes. See [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres).

## Appendix C: Worked examples

- **Individual attestation, self-reported.** Single subject, single signer, `source_type = self_reported`, `witnessing_depth = self_asserted`, `attestor_relationship = self`. Baseline case.
- **ORCID-authored research paper attestation.** Subject is a paper, signer is the author, `source_type = orcid`, `witnessing_depth = computed_match` (name-matching against ORCID record), `attestor_relationship = self`. `confidence = 9500` for the identity match.
- **CRediT contribution role for the same paper.** Subject is the same paper, signer is the same author, `activity_type = credit.niso.org/contributor-roles/data-curation`, `source_type = self_reported`, `witnessing_depth = self_asserted`. `confidence = 8000` (author's own assessment of their role). Illustrates why one paper needs multiple attestations: the ORCID authorship attestation and the CRediT role attestation carry different provenance and different confidence, and combining them into one record would misrepresent either or both.
- **Peer-witnessed contribution.** Subject is a person, signer is a peer, `source_type = peer_witnessed`, `witnessing_depth = physically_observed`, `attestor_relationship = peer`. High trust artifact.
- **Batched attestation (Merkle-root, many subjects).** See §5.2.
- **Additive revocation.** New attestation, `activity_type = sworn.dev/v1/revocation`, `subject = target_attestation_id`, signed by the same signer as the target.
- **Cross-implementation verification.** Postgres-signed attestation verified by a Solana implementation. Signatures match because `canonical_bytes` are identical byte-for-byte; substrate is irrelevant to signature verification.

## Appendix D: Glossary

## Appendix E: Changelog

**v0.1-final** (this document): added `spec_version`, `source_hash`, `source_type`, `confidence`, `witnessing_depth`, `attestor_relationship` to canonical bytes. Extended canonical byte length from 208 to 248. Added §9.2, §9.3, §9.4 registries. Added §2.5 (provenance fields), §2.5.1 (signer's claim not verifier's guarantee), §2.5.2 (dedup semantics), §2.8 (source license and off-chain metadata), §10.4 (reference test vectors). Added the "sourceable" property to §1.1.

**v0.1-preview** (deprecated): initial nine-field attestation record without provenance. Canonical byte length 208. Superseded by v0.1-final during the pre-review window; no external reviewer attested to v0.1-preview bytes.
