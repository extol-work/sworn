# SWORN Implementation Notes

**Status:** informative. These notes describe patterns that make SWORN implementations robust in production without constraining conformance. Nothing in this file affects whether an implementation conforms to the specification (see SPEC.md §10). Implementers building for durable operation over years are the intended audience.

---

## Additive-only migration in production

**The one production constraint that matters most.** Every SWORN storage layer in production carries live attestations that other services depend on. Schema changes made during spec evolution MUST be additive-only. In practice:

- `ADD COLUMN` with a default is safe. It runs in constant time on modern PostgreSQL and does not lock the table.
- `TRUNCATE`, `DELETE FROM ... WHERE`, `DROP TABLE`, and `DROP COLUMN` are unsafe on production data. They destroy attestations that may already have been referenced by external verifiers.
- `UPDATE ... SET` on signed content (`signer`, `subject`, `data_hash`, `source_hash`, `nonce`, `signature`) is unsafe and would invalidate signatures. Even repairs that seem cosmetic risk breaking cross-implementation verification.

The pattern that works for a spec version bump:

1. Add new columns with defaults matching the semantics of pre-migration rows. For the v0.1-preview to v0.1-final transition: `spec_version SMALLINT NOT NULL DEFAULT 1` (marks all existing rows as preview), then flip the default to `2` for new inserts after deploy.
2. Backfill provenance fields for historical rows in a follow-on job, marking them `provenance_origin = 'backfilled'` in off-chain metadata (see SPEC §2.8).
3. Leave old columns in place until nothing reads them. Dropping columns is a separate migration cycle much later, once every dependent service has been retired.

Historical rows with `spec_version = 1` remain verifiable against v0.1-preview canonical bytes forever. Their signatures do not migrate. Do not attempt to re-sign them under v0.1-final; the signer's key is required and re-signing would replace the original attestation act with a synthetic one.

---

## Fail-closed on unknown enum values

The `source_type`, `witnessing_depth`, and `attestor_relationship` fields are enums registered in SPEC §9.2–§9.4. When a reader encounters an unknown value:

- The correct response is **refusal to interpret**, not silent coercion. Do not fold unknown source_type values into `unknown` (0); do not fold unknown witnessing_depth into `unspecified` (0).
- Distinguish the error path clearly. An unknown enum value is a **version-mismatch condition** (this reader is behind the current registry). A malformed attestation is a **corruption condition** (the bytes cannot be parsed at all). Both must be non-fatal for the process but must be reported distinctly so operators can diagnose "reader needs an update" vs "attestation is broken."
- Persist the raw bytes so a later reader with an updated registry can interpret them. Do not drop the row.

This creates an intentional live tension with additive-only migration: an implementation that has updated its registry can produce attestations that older implementations must refuse. The refusal is the mechanism. Silent misinterpretation would be worse.

---

## Provenance snapshot semantics

Two fields on an attestation are snapshots of state at signing time and remain accurate as historical records regardless of subsequent change:

- `attestor_relationship` (SPEC §2.5): the signer's relationship to the subject **at signing time**. A signer's live role in a community may change after the fact (a coordinator steps down, an institution's association ends). The signed attestation preserves the relationship as it existed when the claim was made.
- `confidence` (SPEC §2.5): the signer's estimate of the claim's confidence **at signing time**. If a signer later discovers the claim was weaker than they thought (e.g., an ORCID collision surfaces during dedup), they issue a new attestation with corrected confidence. Do not mutate the historical `confidence` value.

Implementations SHOULD run a nightly integrity check that flags cases where a signed `attestor_relationship` value diverges from the signer's live role at the moment of signing. Divergence over time (role change after the fact) is expected and healthy; divergence at signing time indicates a signing-path bug.

---

## Original vs backfilled provenance

When migrating from v0.1-preview to v0.1-final, historical rows do not carry provenance signed by the original signer. Backfilling those fields is legitimate but epistemically different from provenance produced at the time of the original attestation.

Recommended off-chain metadata field on every stored attestation:

```
provenance_origin: 'original' | 'backfilled'
```

`original` means the provenance fields were produced by the signer at signing time and are covered by the signature. `backfilled` means the provenance fields were produced by an implementation-side migration and are not covered by any signature. Verifiers walking the graph may choose to weight backfilled provenance differently.

For the v0.1-preview to v0.1-final transition specifically: all historical v0.1-preview rows should be marked `backfilled`, because their signatures do not cover the new provenance fields. Only new v0.1-final attestations produced by their signers are `original`.

---

## License metadata as time-versioned records

SPEC §2.8 requires implementations to preserve and propagate license information alongside attestation records. In practice, source license terms change over time (StackOverflow license disputes, GitHub TOS updates, individual OpenAlex-aggregated sources with varying terms). A single-column `license` field loses the audit answer to "was this attestation ingested when the source was still redistributable?"

Recommended storage pattern:

```
attestation_license_history (
  attestation_id,
  license_identifier,
  license_effective_range,   -- e.g., tstzrange
  observed_at
)
```

The pattern preserves license-at-time-of-ingest as auditable data. When a source's license changes, the pipeline records the new terms and effective range without overwriting the historical record.

This is not signed content and does not affect signature verification. It is graph-analysis territory that becomes load-bearing when a verifier needs to answer license-provenance questions years after the fact.

---

## Idempotency and cross-implementation source identity

`source_hash` is deterministic given a valid canonical source identifier per SPEC §9.2. Two implementations ingesting the same ORCID paper, the same DOI, or the same git commit MUST produce the same `source_hash` bytes when both follow the registered `MUST` canonicalization procedure.

For source types with `SHOULD` canonicalization (`rss_parsed`, `open_source_project`, `coordinator_confirmed`, `computed`, `system_observed`), cross-implementation identity is a best-effort convention. Two implementations MAY produce different `source_hash` values for the same underlying source and both remain conforming. Applications requiring cross-implementation graph analysis on these source types SHOULD document their canonicalization procedure so peers can align.

For local idempotency (same signer resubmitting the same attestation), the `unique_attestation` constraint in the reference implementation (`signer + subject + activity_hash + data_hash + nonce`) handles the duplicate case with an HTTP 409 return carrying the existing attestation id. This is a reference-implementation choice, not a spec requirement; other implementations MAY choose different idempotency semantics.

---

## Reference test vectors

The `fixtures/attestations/v0.1-final/` directory in this repository contains golden test vectors that any conforming implementation MUST reproduce byte-for-byte. Vectors specify `input_fields`, `expected_canonical_bytes_hex`, and `expected_signature_hex`.

Implementations discovering edge cases during production use SHOULD contribute additional vectors. The vector set is meant to grow with the specification's operational maturity.

Cross-implementation verification runs the vector reproduction as a pre-flight test: if implementation A and implementation B both reproduce every vector, they will produce identical canonical bytes for identical field sets. If one fails, the diff is byte-visible and the failing implementation has an isolated bug to fix.

---

## When in doubt, err toward additive

The spec was designed to evolve additively wherever possible: new source_type values, new activity types, new registered vocabularies do not advance `spec_version`. Only changes to the canonical byte sequence layout, algorithm changes, or field reorderings advance the version.

Implementations that treat the registries as living documents (fetch updated enum values from a well-known location, degrade gracefully to `version-mismatch` refusal on unknown values, re-sign nothing) will remain conforming across many minor evolutions without operational disruption. Implementations that hard-code the registries at deploy time will need periodic updates but retain full correctness in the interim.
