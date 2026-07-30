# Contributing to SWORN

SWORN is under RFC review during v0.1. Contributions are welcome across several distinct paths.

## Attesting to the spec

The primary form of contribution during the RFC period. See [RFC.md §Attesting to the spec](./RFC.md#attesting-to-the-spec).

If your attestation endorses the spec, disagrees with a section, or proposes an amendment, feel free to open a PR that references it so the discussion has a natural home. Do not submit textual endorsements. Submit signatures.

## Proposing spec changes

Open a PR against `SPEC.md` (or the relevant section file once we split it). Include:

- The change you're proposing
- The rationale (why the current text is insufficient)
- Which implementations you've tested the change against, if any
- Whether the change is backwards-compatible with v0.1 as currently drafted

For substantive changes (anything touching required MUST/MUST NOT/SHOULD language), also open a companion discussion in Issues so the reasoning has a durable home outside PR diff view.

## Reporting spec ambiguity

If you tried to implement SWORN and hit a place where the spec was ambiguous, meaning two conforming implementations would disagree or the spec doesn't say what to do in a specific edge case, open an issue with the `ambiguity` label. Include:

- The exact spec text that was ambiguous
- The two (or more) interpretations you considered
- Which one your implementation chose, and why

Ambiguity reports are the highest-leverage contributions we can receive during RFC review.

## Reference implementation contributions

Not this repo. See [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres) for the reference implementation, and its own CONTRIBUTING.md.

## What we're deliberately not looking for

- Additions to v0.1 scope that could live in a companion spec (see [OPEN_QUESTIONS.md](./OPEN_QUESTIONS.md) for what's deferred and why)
- Bikeshedding on names (the name is `SWORN`; the acronym expands to Signed, Witnessed, Open, Recorded, Non-transferable; these are locked for v0.1)
- Voting math, token economics, or governance weight function proposals (out of spec scope by design)

## Code of conduct

Standard applies: no personal attacks, no bad-faith engagement, no vendor sniping. Disagreement is welcome; disrespect is not. If you're not sure whether something crosses the line, don't send it.

## Provenance and credit

Contributors whose spec changes land in a required section will be credited by public key in the CHANGELOG and, if they wish, by name in a `CONTRIBUTORS.md` file added to the repo. Attestations to the spec speak for themselves. They're already in the graph.
