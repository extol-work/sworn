# HTTP conformance tests (Rust)

Exercises SPEC §10.2 tests T-2, T-3, T-4, T-6, and T-7 against a live
SWORN implementation exposed over HTTP. T-1 and T-5 are covered by the
vector-check runners at `fixtures/runners/`; T-8 (notarizer independent
recomputation) is exercised implicitly by T-3; T-9 (no substrate mutation)
is inspected out-of-band per implementation.

## Usage

```bash
cd fixtures/tests/rust
cargo run --quiet -- http://localhost:8080
```

The URL is required. No default. A runner that silently exercises the
wrong implementation is worse than one that refuses.

## Wire shape

The runner speaks the reference `sworn-postgres` HTTP shape:

- `POST /attestations` for creation (returns 201 on success, 400 on client
  error, 4xx or 5xx per implementation on other errors).
- `GET /attestations` for the refused-enumeration test (T-4).
- `GET /healthz` for the pre-flight reachability check.

Implementations with a different wire shape can port `src/main.rs` while
preserving the per-test assertions. The tested properties (refuse
enumeration, reject tampered signature, reject tampered payload, reject
malformed provenance, reject unknown enum values) are spec requirements
independent of transport.

## Per-test coverage

| Test  | SPEC ref  | What it exercises |
|-------|-----------|-------------------|
| Pre-flight | (n/a) | A well-formed attestation is accepted. Failure means the test fixture is broken; every subsequent result is meaningless. |
| T-4 | §6.4, §10.2 | `GET /attestations` returns 4xx (refused enumeration). |
| T-2 | §3.1, §10.2 | Flipping one byte of the signature causes the server to reject with 400. |
| T-3 | §2.3, §3.1, §10.2 | Sending payload B under an attestation signed over payload A causes the server to independently recompute `data_hash` and reject. Also implicitly exercises T-8. |
| T-6a | §2.4, §10.2 | `source_type = self_reported (1)` with non-zero `source_hash` is rejected at verification. |
| T-6b | §2.4, §10.2 | `source_type = unknown (0)` with non-zero `source_hash` is rejected at verification. |
| T-7a | §9.2, §10.2 | Unknown `source_type` (99, outside 0..=14) is rejected fail-closed. |
| T-7b | §9.3, §10.2 | Unknown `witnessing_depth` (200, outside 0..=5) is rejected fail-closed. |
| T-7c | §9.4, §10.2 | Unknown `attestor_relationship` (200, outside 0..=6) is rejected fail-closed. |

## Deliberate scope

**No dependency on `sworn-postgres` code.** The runner duplicates the
canonical-bytes construction. That's intentional: any drift between the
spec and the reference implementation must surface as a per-test failure,
not a compile-time coupling. Same discipline as the vector runners.

**Per-test diagnostics.** Each test reports pass/fail independently.
Failure messages include the HTTP status received and the first ~200
characters of the response body so the failure mode is legible without
enabling a debug flag.

**No default URL, no default fixture path.** Fail loud rather than
silently exercise the wrong target.

**Fresh keypair per test.** Every test uses its own signer, so a bug in
one test path cannot pollute another test's state. Salted payloads
prevent nonce/idempotency collisions across a re-run.

## What this does NOT test

- **T-1 and T-5** (byte-identical serialization/signing against reference
  vectors): use the vector-check runners at `fixtures/runners/`.
- **T-8** (notarizer independent recomputation): exercised implicitly by
  T-3. Formal test requires substrate-level inspection.
- **T-9** (no substrate mutation): substrate-specific; verify via
  implementation source review, not a runtime probe.
- **Rate limiting behavior (§6.5).** The rate limiter should not affect
  correctness tests; if a rate-limited response is received, it will
  surface as a test failure with the 429 status echoed so you can raise
  the limit or space runs.
- **Disclosure token flow (§6.3).** A conforming implementation offering
  disclosure is expected to pass separately; test coverage lives in the
  reference implementation's own quickstart script.

## Reference implementation

The [`sworn-postgres`](https://github.com/extol-work/sworn-postgres)
reference implementation is expected to pass 9/9. To run against it:

```bash
# Terminal 1: bring up a fresh sworn-postgres instance
cd path/to/sworn-postgres
docker compose up -d          # or run cargo -p sworn-api against local pg

# Terminal 2: run the conformance suite
cd path/to/sworn/fixtures/tests/rust
cargo run --quiet -- http://localhost:8080
```
