# SWORN — Roadmap

Not a promise. A working plan for what lands when. Adjusted as reality intrudes.

## v0.1 — RFC (target: October 2026)

- [ ] §1–2 Overview + Layer 1 (Testimony) normative text
- [ ] §3 Layer 2 (Signing) normative text
- [ ] §4 Layer 3 (Registry) normative text — single signer type
- [ ] §5 Layer 4 (Notarization) normative text — substrate-agnostic
- [ ] §6 Layer 5 (Presentation) normative text — two-call design
- [ ] §7 Security considerations
- [ ] §8 Privacy considerations
- [ ] §9 Registries scaffold
- [ ] §10 Conformance criteria
- [ ] Appendix A — Solana/SAS informative binding
- [ ] Appendix B — Postgres informative binding
- [ ] Appendix C — Worked examples
- [ ] Appendix D — Glossary
- [ ] Reference implementation ready ([extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres))
- [ ] CLI (`sworn`) published
- [ ] Verification endpoint contract (OpenAPI) frozen
- [ ] RFC intro published + reviewer signatures accumulating

## v0.2 — Extensions round (target: Q1 2027)

Companion documents, not core changes:

- [ ] SWORN-Extensions-Roles — two-layer witness/certifier pattern
- [ ] SWORN-Extensions-Affiliation — persona-entity binding + revocation
- [ ] Validator disclosure schema (worked example built on Extensions-Roles)

## v0.3+ — TBD

- Voter delegation
- Multi-signature attestations
- Zero-knowledge disclosure
- Cross-chain notarization proofs

## Ratification

v0.1 is an RFC. There is no formal ratification body. Adoption is the ratification signal — measured by (a) implementations that pass conformance tests, (b) attestations in the wild that reference the spec, (c) reviewers whose signatures on the spec accumulate as its provenance record.

If v0.1 hasn't attracted at least three independent implementations and a substantive body of reviewer attestations by mid-2027, we treat that as a signal to reconsider the design rather than push adoption.
