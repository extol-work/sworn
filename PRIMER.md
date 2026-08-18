# Primer

**Status:** Non-normative. Companion to the Attestation Notary Specification (SPEC.md, draft v0.2).

This document explains what the specification is, why it is shaped the way it is, and how to read it. Nothing here creates conformance requirements. All requirements live in SPEC.md.

Reviewers who want to understand the design should read this first, then SPEC.md. Implementers can skip straight to SPEC.md. Adopters evaluating whether the specification fits their use case should read this, then §1 of SPEC.md, then the bindings.

---

## What this specification is

A specification for signed statements whose existence is timestamped on a public, tamper-evident ledger. A conforming attestation has three properties:

1. Someone with a private key produced it, and the signature can be verified by anyone with the public key.
2. Its hash was published to the ledger at a specific point in time, and the timestamp cannot be forged or backdated after the fact.
3. The public ledger cannot be queried to enumerate a person's signing history or a subject's inbound attestations by scanning it.

The first property is signing. Any signing spec provides this.

The second property is notarization. Certificate transparency logs provide this. Git commit chains provide this. Blockchain timestamps provide this.

The third property is the one that makes the specification worth publishing rather than shipping as a library. Most systems that combine signing and public timestamps produce a searchable index of who signed what about whom, because the substrate they anchor to is a searchable index by construction. The contribution here is a discipline about how to anchor without producing that index, so verification of a specific known attestation works but bulk discovery does not.

## What this specification is not

The specification does not define:

- **A trust model, a reputation score, or a ranking.** The graph of signed attestations is a public artifact. What weight to give any given attestation, how to combine them into a score, and whether to trust the resulting number are choices the reader makes. Two readers looking at the same graph can compute different scores and both be conforming.

- **Identity verification.** A signer is a public key. The mapping between a public key and a person, an organization, or a role is entirely outside the specification. Applications that want signer identity build it as a separate directory, subject to their own privacy and legal constraints.

- **Roles, affiliation, or delegation.** Every signer is a single key. There is no concept of an organizational key delegating signing authority to individuals, no directory of official signers, no cryptographic representation of institutional roles. Applications that need such structure build it out of primitives (typically as pairs of attestations that assert the relationship) without asking the protocol to model it.

- **Downstream governance constraints on standing.** Non-transferability, standing-conversion transparency, and prohibitions on wrapping attestations in tokens are legitimate concerns for adopters building governance or economic systems. The specification does not attempt to enforce them at the byte layer: doing so would either add machinery most adopters do not need or make claims the spec cannot honor (a downstream implementer can wrap an attestation in a token regardless of what the spec says). Adopters document their own commitments; Extol's guardrails, published separately, are one example.

Reviewers familiar with W3C Verifiable Credentials, Solana Attestation Service's native product, or DID-based identity systems will notice this list rules out most of what those systems specify. That is intentional. Those systems are trying to solve identity and role modeling. This specification is trying to solve "signed fact + honest timestamp + non-enumerable substrate," and stops there.

## Neighboring specifications

Several existing specifications address signed factual claims: Verifiable Credentials (W3C), Verifiable Presentations (W3C), Open Badges 3.0 (IMS Global), C2PA (Coalition for Content Provenance and Authenticity), Sigstore (Linux Foundation, primarily for software artifacts). Each solves a real problem inside its target domain. This specification takes a narrower cut: it defines what a signed fact looks like as bytes, what provenance those bytes carry, and how a hash of those bytes is committed to a public substrate that verifiers can read without asking the platform of origin. What the substrate publishes is a payload-excluded, issuer-free envelope and a public hash that is not a directory.

## Non-enumerability

The specification's Layer 5 (Presentation) and Layer 4 (Notarization) both refuse to publish or expose the graph of who signed what about whom in a form that supports bulk queries.

The failure mode this prevents is straightforward. If the notary substrate publishes each attestation's signer public key and subject public key as searchable fields, anyone can walk the substrate and produce "here is everything Alice has ever signed" and "here is everyone who has ever signed anything about Bob." That query is useful for Sybil detection, ring identification, and standing accumulation, and it is also useful for compiling a dossier on a person from public records they never expected to be indexed.

The specification takes the position that the dossier concern is more serious than the graph-analysis concern, and structures the substrate binding to defeat both queries at the cost of making graph analysis a reader-side, per-attestation-known-in-advance operation rather than a scan.

Concretely: the substrate stores an opaque per-attestation hash and a timestamp. Nothing about the signer, subject, activity type, or provenance appears in a form that can be scanned by those attributes. Readers who want to verify a specific attestation they already know about can do so trivially. Readers who want to enumerate must obtain attestations from parties who legitimately hold them (typically the signer or subject) rather than from the substrate.

Non-enumerability is the shorthand used in the specification for this discipline. It appears in SPEC.md at §5 (Notarization) and §6 (Presentation) and, after the definition, is used without further explanation.

## Why Solana

The specification binds notarization normatively to Solana Attestation Service. This is a substrate choice, not an ideological one, and the engineering reasons for it are worth naming so future substrate-porting discussions have concrete criteria to work against.

**SAS exists as a native primitive.** Solana Attestation Service is a first-class program on Solana with credential authorities, schemas, and attestation accounts. Building the equivalent on Ethereum, Cosmos, or Bitcoin would mean writing a custom attestation contract, defining a credential-authority pattern, and shipping indexing tooling. SAS gives us the notary substrate essentially for free. Nothing else in the current ecosystem has this at the same maturity.

**Ed25519 is native.** Solana signs transactions with Ed25519, which matches the mandatory-to-implement algorithm at Layer 2. Zero cross-algorithm bridging. Ethereum uses secp256k1, which would either force the specification to define multiple curves as a permanent implementer burden or require signature bridging that adds verification complexity.

**PDA seeds are caller-chosen.** Solana's Program Derived Address model lets the caller construct the seed. §5.1's non-walkability discipline requires the notary to use an opaque per-attestation hash as the account address rather than any identifying field. Solana's PDA model supports this trivially. Ethereum's account model (address is a function of the private key) does not offer the same freedom to construct addresses that reveal nothing about their contents.

**Cost matches the target volume.** Solana anchors at roughly $0.0006 per attestation and requires no per-account rent for the pure-notary pattern the specification adopts. This makes high-volume observations (per-hour attendance records, weekly digests, per-vote participation records) economically honest. Ethereum L1 at $5+ per attestation would force Merkle batching for everything, which adds proof complexity for every reader. L2s are cheaper than L1 but still 100 to 1000 times more expensive than Solana, and each L2 introduces its own trust assumptions.

**Sub-5-second finality matches the interaction.** "Sign and move on" is the user experience testimony wants. Ethereum's 12-to-15-minute finality window forces asynchronous "we will notify you when it settles" UI that breaks the register of a witnessed statement.

**Precedent exists in production.** The memo-anchoring-on-Solana pattern has cleared App Store review and is running commercially in adjacent applications. Not a novel research substrate.

The binding to Solana is defensible on these criteria, not on tribal loyalty to a particular blockchain community. A future substrate that satisfies all six properties (Ed25519 or cheaply bridged, caller-chosen non-walkable addressing, sub-cent per-attestation cost, sub-minute finality, first-class attestation primitive, production track record) could be adopted through a future binding without changing SPEC.md. None currently qualify. If one does, we add the binding.

## What Extol builds on top

Extol, Inc. is the first production adopter of the specification and the party that authored it. Extol operates:

- A signing surface where community members produce attestations about their contributions and participation. Signers use their own keys, produced through a passkey-derived flow that keeps the signing material off Extol's servers.
- A notarization service that batches and anchors attestation hashes to Solana Attestation Service on mainnet.
- A verification surface where third parties (grant reviewers, hiring managers, coalition partners) can confirm a specific attestation is valid and durable.
- A product surface where the aggregated graph of a person's or a community's attestations is displayed to that person or community as their standing.

The specification does not require or prevent any of this. Another organization could build a completely different product on the same specification. What the specification does is make Extol's attestations verifiable by parties who have no relationship with Extol and no need to trust Extol's servers.

Adopters considering the specification should think about which of the following they need:

- **Verification without platform trust.** A signature that outlives the platform of origin.
- **Timestamped commitment.** A record that cannot be silently rewritten after the fact.
- **Non-surveillance.** A public verification path that does not produce a queryable index of your users' signing history.

If all three are needed, this specification is a reasonable choice. If only the first is needed, a plain signing library will do. If the second is needed but not the third, certificate transparency logs or git-anchored logs work fine.

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

If you are reviewing the specification for a standards body or for your organization's adoption committee, the questions you probably want answered are:

**Is the byte layout final?** For v0.2 with spec_version = 3, yes. Any change to the byte layout advances spec_version to 4 and requires implementations to dispatch on the version marker. The byte layout has been stable across three implementations for several months of production use and has not needed changes.

**Is the SAS binding portable?** No, and this is intentional. Other substrate bindings can be authored as informative alternatives, but Level 3 (Notarizer) conformance in v0.2 requires SAS specifically. This lets the specification define concrete non-walkability rules for the substrate rather than substrate-agnostic aspirations.

**What is the governance path?** Currently: Extol authors changes, changes are reviewed in public, and versions advance when review has settled. No committee, no vote count required. If the specification is ever submitted to a standards body (see below), governance moves to that body.

**What are the publication routes?** Three are worth naming:
- **Self-publication (current).** Extol maintains the specification. Adopters use it under Apache 2.0 license. This is where the spec is today and where it will remain until adoption justifies otherwise.
- **Independent Submissions Editor at the RFC Editor.** A path to publishing as an Informational or Experimental RFC without requiring a working group. Roughly a six-to-twelve-month process with editorial polish and IESG review. Worth considering once three or more independent implementations exist that were not authored by Extol.
- **IETF working group.** A two-to-three-year commitment requiring enough implementer interest to justify forming or joining a working group. Not appropriate until adoption is broad enough to warrant it.

**What is the risk of specification change?** Byte layout: near-zero, changes require a spec_version bump. Registry additions (new source_type, new activity types): common and expected. Prose clarifications: expected. Substantive semantic changes: rare and gated by review.

## Note on the deferred signature-collection surface

Earlier drafts described a signature-collection surface where reviewers could sign an attestation whose subject was a specific commit hash of the specification, as a "prove the spec by using the spec" review mechanism. That surface is deferred in v0.2.

Two reasons. First, the surface is chicken-and-egg at launch: a signer would be signing against text they cannot yet cryptographically conform to, because the reference implementation stack has not yet aligned with v0.2. Aspirational calls-to-action leak "we did not finish this" and are worse than absent ones. Second, the audience the mechanism was designed for (adopters willing to publicly commit to accepting attestations under this specification) is better served by an adoption-ledger artifact hosted separately from the specification repository, where the semantic weight of a signature is "we will act on these attestations" rather than "we endorse the current draft text." Endorsement of text is what GitHub PRs and issues are for.

The mechanism is deferred, not cancelled. If a body of adopters emerges who want a durable signed record of their commitment to the specification, a future revision may reintroduce a signature-collection surface with that shape. Nothing in v0.2 forecloses it.

## Historical note on the SWORN name

Prior versions of this material were published under the name SWORN, which expanded to Signed, Witnessed, Owned, Recorded, Notarized. The v0.2 rewrite retired the name and the letter-per-property expansion because the protocol only specifies Signed and Notarized. Witnessing is a pointer field, not a protocol operation. Ownership and non-transferability are downstream governance concerns adopters implement in their own layers, not properties the spec attempts to establish. Recording is a subset of notarization. Retaining an acronym that promised properties the bytes could not deliver was inviting readers to look for a shape the specification did not have.

The GitHub repository retains the `sworn` short name for URL stability; the specification itself is untitled beyond its descriptive name and its draft version marker.

## Version history

**v0.2.1 (2026-08-17, errata pass).** External review surfaced three areas where the v0.2 text overreached; this errata addresses them without a spec_version bump because the 248-byte canonical layout is unchanged. The substantive shifts are: the Layer 5 identifier separates from the on-chain `attestation_hash` and becomes an independent random per-attestation `disclosure_identifier` (closing the "scrape the substrate, verify each hash" enumeration path); SPEC §5.4 reframes durability as an operator commitment above SAS rather than a substrate-provided invariant (SAS's `CloseAttestation` remains reachable by the credential authority); and §10 retreats from "Conformance" framing to "Implementation checklists" to match the fact that the spec does not yet carry an independent test suite or certification process (every byte-level and algorithm-level MUST inside the checklists survives). Smaller corrections in bindings/sas.md (expiry MUST, cost honesty, explicit schema layout tags), §9.3/§9.4 (cross-field verifier MUSTs), and §8.1/§9.2 (source_hash reversibility) travel with the errata.

**v0.2 (2026-08-12).** Rewrite. Notarization becomes SAS-normative. Non-enumerability becomes the central discipline at Layer 5. The former §1.5 non-transferability firewall retired; downstream governance constraints on standing are now scoped as adopter concerns rather than protocol properties. Witnessing removed as a protocol operation; `witness_for` remains as a pointer field. `created_at` renamed to `signer_asserted_at` and substrate time made authoritative. Nonce derivation fixed so same-subject, different-payload attestations do not collide. Revocation subject convention unified on SHA-256 of target canonical bytes. `sworn.dev/v1/revocation` registered. CRediT URIs corrected to include scheme and trailing slash per NISO. Extol/Cortex references swept from normative text. Structural split: SPEC.md purely normative, PRIMER.md non-normative rationale, bindings/sas.md and bindings/postgres.md as concrete substrate bindings. spec_version advances 2 to 3. The SWORN name retired from prose. RFC.md and the signature-collection surface deferred. QUICKSTART.md removed; sworn-postgres deprecated. Front-facing not-in-scope lists shortened to items readers would reasonably expect the spec to address (ranking, identity, roles).

**v0.1-final.** Added five provenance fields to the canonical byte sequence: source_hash, source_type, confidence, witnessing_depth, attestor_relationship. Prefixed byte sequence with an explicit spec_version marker. Byte length grew from 208 to 248 bytes. spec_version 1 to 2.

**v0.1-preview.** Initial draft. Not conforming to any later version.

Pre-v0.2 material is preserved in git history rather than in a separate archive directory.
