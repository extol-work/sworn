# SWORN Primer

**Status:** Non-normative. Companion to SPEC.md v0.2.

This document explains what SWORN is, why it is shaped the way it is, and how to read the specification. Nothing here creates conformance requirements. All requirements live in SPEC.md.

Reviewers who want to understand the design should read this first, then SPEC.md. Implementers can skip straight to SPEC.md. Adopters evaluating whether SWORN fits their use case should read this, then §1 of SPEC.md, then the bindings appendix.

---

## What SWORN is

SWORN is a specification for signed statements whose existence is timestamped on a public, tamper-evident ledger. A conforming SWORN attestation has three properties:

1. Someone with a private key produced it, and the signature can be verified by anyone with the public key.
2. Its hash was published to the ledger at a specific point in time, and the timestamp cannot be forged or backdated after the fact.
3. The public ledger cannot be queried to enumerate a person's signing history or a subject's inbound attestations by scanning it.

The first property is signing. Any signing spec provides this.

The second property is notarization. Certificate transparency logs provide this. Git commit chains provide this. Blockchain timestamps provide this.

The third property is the one that makes SWORN worth publishing as a specification rather than as a library. Most systems that combine signing and public timestamps produce a searchable index of who signed what about whom, because the substrate they anchor to is a searchable index by construction. SWORN's contribution is a discipline about how to anchor without producing that index, so verification of a specific known attestation works but bulk discovery does not.

## What SWORN is not

SWORN does not specify:

- **A witnessing operation.** When one party signs a statement about another party, the other party does not sign anything as a consequence. The `witness_for` field in an attestation is a pointer, not an operation. Applications that need a real two-party witness pattern build it as a pair of attestations, not as a SWORN primitive.

- **Non-transferability.** A signed statement is a copyable object. If a signing key is sold, the buyer signs valid attestations under the seller's public key indistinguishable from the seller's own. SWORN cannot prevent this at the protocol layer and does not try. Applications that want the appearance of non-transferability build it as product policy above SWORN, not as a SWORN property.

- **A trust model, a reputation score, or a ranking.** The graph of signed attestations is a public artifact. What weight to give any given attestation, how to combine them into a score, and whether to trust the resulting number are choices the reader makes. Two readers looking at the same graph can compute different scores and both be conforming.

- **Identity verification.** A SWORN signer is a public key. The mapping between a public key and a person, an organization, or a role is entirely outside the specification. Applications that want signer identity build it as a separate directory, subject to their own privacy and legal constraints.

- **Roles, affiliation, or delegation.** Every signer is a single key. There is no concept of an organizational key delegating signing authority to individuals, no directory of official signers, no cryptographic representation of institutional roles. Applications that need such structure build it out of SWORN primitives (typically as pairs of attestations that assert the relationship) without asking the protocol to model it.

Reviewers familiar with W3C Verifiable Credentials, Solana Attestation Service's native product, or DID-based identity systems will notice this list rules out most of what those systems specify. That is intentional. Those systems are trying to solve identity and role modeling. SWORN is trying to solve "signed fact + honest timestamp + non-enumerable substrate," and stops there.

## The name

SWORN originally expanded to Signed, Witnessed, Owned, Recorded, Notarized. In v0.2 the letter-per-property expansion is retired from normative text because the protocol only specifies Signed and Notarized. Witnessing is not a protocol operation. Ownership and non-transferability are not enforceable at Layer 1. Recording is a subset of notarization.

The project keeps the name SWORN because the trademark is filed under it and the RFC review is under way under it. When and if the specification is submitted to a standards body (see below), a numbered-draft rename can happen at that point without disturbing the current project identity.

Historical origin is documented here so anyone reading the archive of pre-v0.2 material can trace what changed. The archive is at `../archive/` in the working repository.

## Why non-enumerability matters

The specification's Layer 5 (Presentation) and Layer 4 (Notarization) both refuse to publish or expose the graph of who signed what about whom in a form that supports bulk queries. This discipline is what separates SWORN from a public database of signed statements.

The failure mode this prevents is straightforward. If the notary substrate publishes each attestation's signer public key and subject public key as searchable fields, anyone can walk the substrate and produce "here is everything Alice has ever signed" and "here is everyone who has ever signed anything about Bob." That query is useful for Sybil detection, ring identification, and standing accumulation, and it is also useful for compiling a dossier on a person from public records they never expected to be indexed.

SWORN takes the position that the dossier concern is more serious than the graph-analysis concern, and structures the substrate binding to defeat both queries at the cost of making graph analysis a reader-side, per-attestation-known-in-advance operation rather than a scan.

Concretely: the substrate stores an opaque per-attestation hash and a timestamp. Nothing about the signer, subject, activity type, or provenance appears in a form that can be scanned by those attributes. Readers who want to verify a specific attestation they already know about can do so trivially. Readers who want to enumerate must obtain attestations from parties who legitimately hold them (typically the signer or subject) rather than from the substrate.

Non-enumerability is the shorthand for this discipline. It appears in SPEC.md at §5 (Notarization) and §6 (Presentation). After the definition it is used without further explanation.

## Why we don't specify witnessing

An earlier draft of SWORN tried to make witnessing a protocol operation. The design that emerged had a `witness_for` field pointing at another party's public key or another attestation's hash, and prose describing what it meant for one party to witness another.

The problem is that the party being witnessed does not sign anything. A signer can put any pubkey in `witness_for` and the substrate cannot check whether that party actually corroborated the claim. Calling this a witnessing operation overpromised what the bytes could deliver.

The v0.2 posture is honest: `witness_for` is a pointer. If Alice puts Bob's pubkey in `witness_for`, that is Alice's claim about Bob, not Bob's action. Applications that need real two-party witnessing build it as two separate attestations, where Bob signs a second attestation whose subject is the hash of Alice's original. This is a pattern applications can implement freely; SWORN does not require it and does not prevent it.

Witnessing as a social act, ritual, or product experience is not a SWORN concern. That is where applications like Extol's live.

## Why we don't specify non-transferability

An earlier draft of SWORN required implementations to document any conversion of attestation-derived standing into transferable value. This was an attempt to make non-transferability enforceable through transparency.

The problem is that hostile implementations will not document, and lazy implementations will not either. A MUST clause a verifier cannot check against a hostile implementation is a spec smell: it creates the feeling of safety without the substance.

The v0.2 posture is honest: SWORN does not specify an asset, an owner field, or a transfer instruction. Any implementation that presents attestations as tradeable objects is doing something SWORN has no opinion on, because SWORN is a fact-signing spec, not a value-transfer spec. Applications that want durable non-transferability build it above the protocol, typically by not exposing the signing key to users and by refusing to build the transfer instruction. This is Extol's choice and can be other adopters' choice; it does not need to be a MUST in the specification.

## Why Solana

SWORN v0.2 binds notarization normatively to Solana Attestation Service. This is a substrate choice, not an ideological one, and the engineering reasons for it are worth naming so future substrate-porting discussions have concrete criteria to work against.

**SAS exists as a native primitive.** Solana Attestation Service is a first-class program on Solana with credential authorities, schemas, and attestation accounts. Building the equivalent on Ethereum, Cosmos, or Bitcoin would mean writing a custom attestation contract, defining a credential-authority pattern, and shipping indexing tooling. SAS gives us the notary substrate essentially for free. Nothing else in the current ecosystem has this at the same maturity.

**Ed25519 is native.** Solana signs transactions with Ed25519, which matches SWORN's mandatory-to-implement algorithm at Layer 2. Zero cross-algorithm bridging. Ethereum uses secp256k1, which would either force SWORN to specify multiple curves as a permanent implementer burden or require signature bridging that adds verification complexity.

**PDA seeds are caller-chosen.** Solana's Program Derived Address model lets the caller construct the seed. SWORN's §5.1 non-walkability discipline requires the notary to use an opaque per-attestation hash as the account address rather than any identifying field. Solana's PDA model supports this trivially. Ethereum's account model (address is a function of the private key) does not offer the same freedom to construct addresses that reveal nothing about their contents.

**Cost matches the target volume.** Solana anchors at roughly $0.0006 per attestation and requires no per-account rent for the pure-notary pattern SWORN adopts. This makes high-volume observations (per-hour attendance records, weekly digests, per-vote participation records) economically honest. Ethereum L1 at $5+ per attestation would force Merkle batching for everything, which adds proof complexity for every reader. L2s are cheaper than L1 but still 100 to 1000 times more expensive than Solana, and each L2 introduces its own trust assumptions.

**Sub-5-second finality matches the interaction.** "Sign and move on" is the user experience testimony wants. Ethereum's 12-to-15-minute finality window forces asynchronous "we will notify you when it settles" UI that breaks the register of a witnessed statement.

**Precedent exists in production.** The memo-anchoring-on-Solana pattern has cleared App Store review and is running commercially in adjacent applications. Not a novel research substrate.

SWORN's binding to Solana is defensible on these criteria, not on tribal loyalty to a particular blockchain community. A future substrate that satisfies all six properties (Ed25519 or cheaply bridged, caller-chosen non-walkable addressing, sub-cent per-attestation cost, sub-minute finality, first-class attestation primitive, production track record) could be adopted through a future binding without changing SPEC.md. None currently qualify. If one does, we add the binding.

## What Extol builds on top

Extol, Inc. is the first production adopter of SWORN and the party that authored the specification. Extol operates:

- A signing surface where community members produce SWORN attestations about their contributions and participation. Signers use their own keys, produced through a passkey-derived flow that keeps the signing material off Extol's servers.
- A notarization service that batches and anchors attestation hashes to Solana Attestation Service on mainnet.
- A verification surface where third parties (grant reviewers, hiring managers, coalition partners) can confirm a specific attestation is valid and durable.
- A product surface where the aggregated graph of a person's or a community's attestations is displayed to that person or community as their standing.

The specification does not require or prevent any of this. Another organization could build a completely different product on the same specification. What the specification does is make Extol's attestations verifiable by parties who have no relationship with Extol and no need to trust Extol's servers.

Adopters considering building on SWORN should think about which of the following they need:

- **Verification without platform trust.** A signature that outlives the platform of origin.
- **Timestamped commitment.** A record that cannot be silently rewritten after the fact.
- **Non-surveillance.** A public verification path that does not produce a queryable index of your users' signing history.

If all three are needed, SWORN is a reasonable choice. If only the first is needed, a plain signing library will do. If the second is needed but not the third, certificate transparency logs or git-anchored logs work fine.

## How to read the specification

SPEC.md is organized in five layers:

1. **Layer 1, Testimony (§2).** The structure of an attestation record. Fields, hashes, provenance, timestamps.
2. **Layer 2, Signing (§3).** How an attestation is bound to a signer. Canonical bytes, Ed25519, nonce derivation.
3. **Layer 3, Registry (§4).** Signer identity semantics, revocation by additive attestation, standing as a graph.
4. **Layer 4, Notarization (§5).** How attestation hashes are anchored to Solana Attestation Service.
5. **Layer 5, Presentation (§6).** How verifiers request and receive attestations; the shown-not-pulled discipline.

Then:

- **§7, Security considerations.** Named limitations, threat model, what is out of scope.
- **§8, Privacy considerations.** The payload split, subject-mediated disclosure, right-to-forget tension.
- **§9, Registries.** Activity type namespace, source_type, witnessing_depth, attestor_relationship, signature algorithms, substrates.
- **§10, Conformance.** Three levels (Verifier, Signer, Notarizer), interoperability tests, reference vectors.
- **Appendix A, B, C.** SAS binding (normative), sworn-postgres binding (informative), worked examples.

Read §1 for the summary, §2 and §3 for the byte-level rules, then jump to the layer you care about.

## For implementers

Producing conforming attestations requires:

- Constructing the 248-byte canonical byte sequence per §3.1, exactly.
- Signing it with Ed25519 PureEdDSA per §3.2, not Ed25519ph.
- Deriving the nonce per §3.4 such that same-signer, same-subject, different-payload attestations produce distinct nonces.
- Populating provenance fields per §2.5 honestly, including the zero-hash rule for sourceless attestations.
- Publishing the hash to Solana Attestation Service per bindings/sas.md if Layer 4 conformance is required.

Verifying attestations requires:

- Reconstructing the canonical bytes per §3.1 from stored fields.
- Verifying the Ed25519 signature per §3.2.
- Recomputing SHA-256 of the canonicalized payload and confirming equality with data_hash.
- Refusing to enumerate per §6.4.

The reference test vectors at §10.4 are the interoperability anchor. If your implementation produces bytes that match vector 1 through vector 5, you produce conforming attestations. If your implementation verifies those five vectors as valid, you verify conforming attestations.

The three implementations currently maintained (Extol's Rust binding of SAS, Titania's TypeScript runner, the standalone Rust runner in fixtures/runners) all agree byte-for-byte on the reference vectors. New implementations that reproduce this agreement can self-declare Level 2 (Signer) or Level 1 (Verifier) conformance per §10.

## For reviewers

If you are reviewing SWORN for a standards body or for your organization's adoption committee, the questions you probably want answered are:

**Is the byte layout final?** For v0.2 with spec_version = 3, yes. Any change to the byte layout advances spec_version to 4 and requires implementations to dispatch on the version marker. The byte layout has been stable across three implementations for several months of production use and has not needed changes.

**Is the SAS binding portable?** No, and this is intentional. Other substrate bindings can be authored as informative alternatives, but Level 3 (Notarizer) conformance in v0.2 requires SAS specifically. This lets us specify concrete non-walkability rules for the substrate rather than substrate-agnostic aspirations.

**What is the governance path?** Currently: Extol authors changes, changes are reviewed in public, and versions advance when review has settled. No committee, no vote count required. If SWORN is ever submitted to a standards body (see below), governance moves to that body.

**What are the publication routes?** Three are worth naming:
- **Self-publication (current).** Extol maintains the specification. Adopters use it under Apache 2.0 license. This is where SWORN is today and where it will remain until adoption justifies otherwise.
- **Independent Submissions Editor at the RFC Editor.** A path to publishing SWORN as an Informational or Experimental RFC without requiring a working group. Roughly a six-to-twelve-month process with editorial polish and IESG review. Worth considering once we have three or more independent implementations not authored by Extol.
- **IETF working group.** A two-to-three-year commitment requiring enough implementer interest to justify forming or joining a working group. Not appropriate until adoption is broad enough to warrant it.

**What is the risk of specification change?** Byte layout: near-zero, changes require a spec_version bump. Registry additions (new source_type, new activity types): common and expected. Prose clarifications: expected. Substantive semantic changes: rare and gated by review.

## Version history

**v0.2 (2026-08-12).** Rewrite. Notarization becomes SAS-normative. Non-enumerability becomes the central discipline at Layer 5. Non-transferability firewall (former §1.5) deleted as non-goal. Witnessing removed as a protocol operation. `created_at` renamed to `signer_asserted_at` and substrate time made authoritative. Nonce derivation fixed so same-subject, different-payload attestations do not collide. Revocation subject convention unified on SHA-256 of target canonical bytes. `sworn.dev/v1/revocation` registered. CRediT URIs corrected to include scheme and trailing slash per NISO. Extol/Cortex references swept from normative text. Structural split: SPEC.md purely normative, PRIMER.md non-normative rationale, bindings/sas.md and bindings/postgres.md as concrete substrate bindings. spec_version advances 2 to 3.

**v0.1-final (2026-XX-XX).** Added five provenance fields to the canonical byte sequence: source_hash, source_type, confidence, witnessing_depth, attestor_relationship. Prefixed byte sequence with an explicit spec_version marker. Byte length grew from 208 to 248 bytes. spec_version 1 to 2.

**v0.1-preview.** Initial draft. Not conforming to any later version. Archived.

Pre-v0.2 material is preserved at `../archive/` in the working repository. Anyone tracking what changed between v0.1-final and v0.2 can diff the two SPEC.md files there.
