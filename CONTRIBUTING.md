# Contributing

This specification is under review at draft v0.2. Contributions are welcome across several paths.

## Proposing changes to the specification

Open a PR against `SPEC.md` (normative) or `PRIMER.md` (rationale). Include:

- The change you are proposing.
- The rationale (why the current text is insufficient or incorrect).
- Which implementations you have tested the change against, if any.
- Whether the change is backwards-compatible with the current byte layout at `spec_version = 3`.

For substantive changes (anything touching MUST/MUST NOT/SHOULD language), also open a companion discussion in Issues so the reasoning has a durable home outside PR diff view. Changes that advance `spec_version` are gated on cross-implementation review.

## Reporting spec ambiguity

If you tried to implement the specification and found a place where the text was ambiguous (two conforming implementations would disagree, or the specification does not say what to do in a specific edge case), open an issue with the `ambiguity` label. Include:

- The exact spec text that was ambiguous.
- The two or more interpretations you considered.
- Which one your implementation chose, and why.

Ambiguity reports are the highest-leverage contribution during the review period.

## Reference implementations

The specification is anchored by cross-implementation vectors in `fixtures/`. Three implementations currently agree byte-for-byte on those vectors. A new implementation demonstrating agreement on the vectors, in a language not yet represented, is a valuable contribution.

Implementation code lives in its own repository. See [extol-work/sworn-postgres](https://github.com/extol-work/sworn-postgres) for the Postgres binding and its own CONTRIBUTING.md.

## What this repository does not accept

- Additions to v0.2 scope that could live in a companion specification. See [OPEN_QUESTIONS.md](./OPEN_QUESTIONS.md) for what has been deferred and why.
- Naming discussions. See PRIMER's historical note on the retired SWORN name.
- Voting math, token economics, or governance weight function proposals. Out of scope by design.

## Code of conduct

Standard applies: no personal attacks, no bad-faith engagement, no vendor sniping. Disagreement is welcome; disrespect is not. If you are not sure whether something crosses the line, do not send it.

## Provenance and credit

Contributors whose changes land in normative text will be credited by public key in the changelog and, if they wish, by name in a `CONTRIBUTORS.md` file.
