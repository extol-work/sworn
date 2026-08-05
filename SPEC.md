# SWORN Specification v0.1 (Draft)

**Status:** DRAFT. Not stable. Not soliciting external review until the reference implementation exercises the text and surfaces the gaps.

§1 through §3 have required text drafted from paper. §4 through §10 are section outlines pending drafts that will be written from working code, not from prose.

**Notation:** The words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are to be interpreted as described in RFC 2119.

---

## §1 Overview

### §1.1 Purpose

SWORN specifies how testimony (the small, corroborable claims that used to live in guilds, unions, and professional societies) can be captured cryptographically so that anyone can verify it later, from anywhere, without needing to trust the platform where it was originally recorded.

This document defines version 0.1 of the specification. It is deliberately minimal. Multi-signer role patterns, delegation, and cross-chain notarization are reserved for future versions (see [OPEN_QUESTIONS.md](./OPEN_QUESTIONS.md)).

### §1.2 Terminology

**Attestation.** A signed statement by one party (the *signer*) about another party or artifact (the *subject*). The statement is captured as a fixed-length hash of a payload, together with metadata that describes what class of statement is being made and when.

**Signer.** The public key that produced an attestation's signature. Standing accrues to the signer, not to the person or organization that operates it. Signers are persistent (the same key over time), but the mapping between a signer and any real-world identity is implementation-defined.

**Subject.** The entity being attested about. May be another SWORN signer, an arbitrary public key, or a content hash. Subject semantics are activity-type-defined.

**Payload.** The semantic content of the attestation. Free-form structured data whose shape is determined by the activity type. The payload is NOT committed on-chain; only its hash is (§2.4, §5.1).

**Activity type.** A URI naming the class of claim being made (e.g., `work.extol.attestation/v1/contribution`, `sworn.dev/validator/v1/block-building-disclosure`). Activity types define the payload schema.

**Notarization substrate.** The public, tamper-evident ledger where attestation hashes are committed. May be a blockchain, a git-anchored append-only log, a certificate transparency log, or any equivalent system. Substrate choice is implementation-defined (§5.1); interoperability is achieved through the substrate registry (§9.3).

**Standing.** The accumulated history of attestations to or by a signer. Standing is NOT a score; it is a graph. Standing MUST NOT be transferable (§1.5). Any conversion of standing into transferable value is a product-layer concern subject to the transparency requirement in §1.5.

**Conforming implementation.** Any software that produces, stores, and verifies attestations per this specification. See §10 for conformance criteria.

### §1.3 Layer model

SWORN is organized in five layers:

- **Layer 1, Testimony (§2):** the structure of an attestation record
- **Layer 2, Signing (§3):** how an attestation is bound to a signer
- **Layer 3, Registry (§4):** signer identity semantics and revocation
- **Layer 4, Notarization (§5):** how attestation hashes are committed to a public ledger
- **Layer 5, Presentation (§6):** how third parties verify attestations

Layers may be implemented independently, but a conforming implementation MUST implement all five. Implementations MAY use any substrate for Layer 4 (§5.1) and any presentation contract that satisfies §6's constraints.

### §1.4 Notational conventions

The words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in this document are to be interpreted as described in RFC 2119.

Byte-level fields are described in little-endian byte order unless explicitly noted. All hashes are SHA-256 (per FIPS 180-4) unless otherwise specified. All signatures are Ed25519 (per RFC 8032, §3.2) unless a registered alternative algorithm is used (§3.3).

Public keys are 32-byte Ed25519 encoded points. Signatures are 64 bytes. Byte concatenation is denoted `||`.

### §1.5 Non-transferability firewall (normative)

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
| `signer` | Ed25519 public key | 32 bytes | Produced the signature. See §3. |
| `subject` | pubkey or content hash | 32 bytes | Entity being attested about. See §2.5. |
| `activity_type` | URI (UTF-8) | variable | Names the class of claim. See §2.2. |
| `data_hash` | SHA-256 | 32 bytes | Hash of the canonical payload. See §2.3–§2.4. |
| `created_at` | int64 Unix seconds | 8 bytes | When the attestation was signed. See §2.6. |
| `retention_hint` | int64 | 8 bytes | Payload retention hint. See §2.6. |
| `witness_for` | pubkey OR 32 zero bytes | 32 bytes | Optional endorsement target. See §2.5. |
| `nonce` | opaque 32 bytes | 32 bytes | Deterministic uniqueness. See §3.4. |
| `signature` | Ed25519 signature | 64 bytes | Over canonical bytes. See §3.1. |

This tuple is the canonical form of an attestation. Serialization for storage and transport is implementation-defined; a conforming implementation MUST be able to reconstruct the canonical byte sequence (§3.1) from stored data in order to verify the signature.

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

### §2.4 activity_hash and data_hash primitives

Two hashes appear in the canonical byte sequence for signing:

- **`activity_hash`** (32 bytes): `SHA-256(UTF-8 bytes of the activity_type URI, after Unicode NFC normalization)`. This gives a fixed-length representation of the variable-length activity type URI.

- **`data_hash`** (32 bytes): `SHA-256(canonicalized JSON payload)` per §2.3. This commits to the semantic content of the attestation without requiring the payload to be transmitted, stored publicly, or preserved indefinitely.

Implementations MUST use SHA-256 for both. Additional hash algorithms may be registered in future versions of this specification.

### §2.5 Subject and witness_for fields

The **`subject`** field is 32 bytes interpreted as one of:

- an Ed25519 public key of another SWORN signer;
- a content hash (32 bytes of hash output, per the activity type's schema);
- an equivalent 32-byte identifier defined by the activity type's schema.

The interpretation is activity-type-defined and MUST be documented in the activity type's schema.

The **`witness_for`** field is always present in the canonical byte sequence (§3.1) and carries one of:

- **32 zero bytes:** the attestation stands alone. The signer is making a first-order claim about the subject.
- **a 32-byte pubkey or content hash:** the attestation endorses or witnesses a specific other party's claim.

`witness_for` enables a signer to say "I saw X's attestation about Y and I corroborate it," without requiring the two-layer role machinery reserved for future versions. Semantics beyond "this signer corroborates that party's claim about this subject" are activity-type-defined.

### §2.6 Timestamp and retention_hint

The **`created_at`** field is a Unix epoch timestamp in seconds (int64), representing when the signer produced the attestation. Signers MUST NOT backdate attestations. Verifiers MAY reject attestations whose `created_at` is more than 300 seconds in the future relative to their local clock, to account for clock skew.

The **`retention_hint`** field is an int64 encoding the signer's intent regarding payload storage duration:

- **`retention_hint > 0`:** Unix epoch seconds after which the payload MAY be discarded by any party storing it. The hash on the ledger remains durable; the payload is not guaranteed to be retrievable after this time.
- **`retention_hint == 0`:** no expiry intent expressed; the implementation-defined default applies.
- **`retention_hint == -1`:** the signer intends the payload to be preserved indefinitely (subject to any implementation's storage limitations).

Retention is a hint, not a guarantee. Verifiers MUST NOT assume payload availability at any future time. The on-chain hash remains verifiable regardless of payload availability (§5.4).

Implementations MAY offer different retention hints as a service tier. If they do, the tiering scheme is subject to §1.5's transparency requirement to the extent that retention differences are exchanged for value.

---

## §3 Layer 2: Signing

### §3.1 Canonical byte sequence for signing

Every signature covers the following byte sequence, in this order:

```
canonical_bytes =
      signer                (32 bytes)
   || subject               (32 bytes)
   || activity_hash         (32 bytes)     -- per §2.4
   || data_hash             (32 bytes)     -- per §2.4
   || witness_for           (32 bytes)     -- per §2.5
   || created_at            (8 bytes, int64 little-endian)
   || retention_hint        (8 bytes, int64 little-endian)
   || nonce                 (32 bytes)     -- per §3.4
```

Total length: **208 bytes** (32 × 6 = 192 for the six 32-byte fields, plus 8 × 2 = 16 for the two 8-byte fields).

Implementations MUST construct this exact byte sequence and MUST NOT include additional fields, framing, prefix bytes, or version markers in the signed content. Additional metadata that an implementation wishes to include in its storage or transport layer MUST NOT enter the canonical byte sequence, otherwise cross-implementation verification breaks.

The signature is `signature = Ed25519.sign(signer_privkey, canonical_bytes)` per §3.2, or the equivalent for a registered alternative algorithm (§3.3).

**Verification procedure:**

1. Reconstruct `canonical_bytes` from the stored attestation.
2. Verify `signature` against `canonical_bytes` using `signer` as the public key, per RFC 8032 §5.1.7.
3. If step 2 succeeds, the attestation is *authentic*: the holder of the `signer` private key produced this exact byte sequence.

Verification per §3.1 does NOT establish that any given payload matches `data_hash`. To verify the payload as well, the verifier MUST also compute `SHA-256(canonicalize(payload)) == data_hash` per §2.3–§2.4.

### §3.2 Ed25519 (mandatory-to-implement)

Every conforming implementation MUST support Ed25519 signing and verification per RFC 8032, with the following parameter choices:

- Curve: Ed25519 (edwards25519), per RFC 8032 §5.1.
- Key encoding: 32-byte public key, 32-byte private seed, 64-byte signature.
- Message: the 208-byte `canonical_bytes` sequence from §3.1, passed to `Ed25519.sign` without pre-hashing (i.e., PureEdDSA per RFC 8032 §5.1, not Ed25519ph).

Implementations MUST use PureEdDSA. Implementations MUST NOT use Ed25519ph.

### §3.3 Signature algorithm extension mechanism

Future versions of this specification MAY register additional signature algorithms. For v0.1, only Ed25519 is defined. Any implementation that encounters an attestation whose algorithm is not Ed25519 MUST reject it as unverifiable. Implementations MUST NOT pass such attestations through as verified.

Because v0.1 defines a single algorithm, no algorithm identifier appears in the canonical byte sequence. Future versions that add algorithms will introduce a prefix byte or equivalent; that will constitute a breaking change (v1.0), not a minor revision. Implementations MUST NOT invent algorithm identifier bytes on their own.

### §3.4 Replay protection (nonce derivation)

The **`nonce`** field is a 32-byte value present in the canonical byte sequence (§3.1) that ensures a signer producing two attestations with otherwise identical fields still produces two distinct signatures over two distinct byte sequences.

The nonce is REQUIRED. The method used to derive it is implementation-defined, subject to the following normative requirements:

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

For v0.1, implementations SHOULD assume signer keys are long-lived. Future versions will define normative rotation patterns.

---

## §4 Layer 3: Registry

### §4.1 Signer identity model (single signer type, v0.1)
### §4.2 Persistent key semantics
### §4.3 Revocation by additive attestation
### §4.4 Standing as an emergent property of the graph
### §4.5 What is NOT in this layer (roles, affiliation, delegation; reserved for future)

---

## §5 Layer 4: Notarization

### §5.1 Hash-anchor commitment (substrate-agnostic)
### §5.2 Merkle batching
### §5.3 Retention semantics (per-record, differing retention allowed)
### §5.4 Durability guarantees for the on-chain hash
### §5.5 Off-chain payload storage (implementation-defined)
### §5.6 What is NOT in this layer (specific chain choice, PDA layouts; see appendices)

---

## §6 Layer 5: Presentation

### §6.1 Verification endpoint contract
### §6.2 Two-call design (verify metadata / disclose payload with subject consent)
### §6.3 Disclosure token semantics
### §6.4 Refused operations (list-by-subject, bulk export, name search)
### §6.5 Rate limiting and abuse considerations

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
| `writing-original-draft` | Writing – original draft | Preparation, creation, and presentation of published work, specifically writing the initial draft. |
| `writing-review-editing` | Writing – review & editing | Critical review, commentary, or revision of published work. |

**Degree of contribution.** CRediT allows an optional degree qualifier of `lead`, `equal`, or `supporting` when multiple contributors share the same role. Implementations expressing this SHOULD carry the degree in the attestation payload (not in the activity type URI). Example payload field: `"contribution_degree": "lead"`.

**Attribution note.** SWORN reserves the `credit.niso.org/contributor-roles/` namespace for interoperability. The CRediT taxonomy itself is owned and maintained by NISO. Implementations using these URIs are consuming CRediT, not extending it.

### §9.2 Signature algorithm registry
### §9.3 Notarization substrate registry

---

## §10 Conformance

### §10.1 Conformance levels
### §10.2 Interoperability tests
### §10.3 Registration process (or self-declaration during RFC period)

---

## Appendix A: Solana / SAS binding (informative)

Extol's on-chain implementation notes. First production adopter. Not normative.

## Appendix B: Postgres binding (informative)

Reference implementation notes. See [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres).

## Appendix C: Worked examples

- Individual attestation (single subject, single signer)
- Batched attestation (Merkle-root, many subjects)
- Additive revocation
- Cross-implementation verification (Postgres signer verified by Solana implementation)

## Appendix D: Glossary

## Appendix E: Changelog
