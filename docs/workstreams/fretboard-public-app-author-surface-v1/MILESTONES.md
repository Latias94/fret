# Fretboard Public App-Author Surface v1 — Milestones

## Milestone 0 — Product taxonomy frozen

Exit criteria:

- the lane records the current public/private split,
- first-open docs state that `fretboard` already owns `new`,
- and the team has one written rule for “public app-author command” vs “repo maintainer command”.

Current state:

- reached on 2026-04-09 for the initial taxonomy snapshot and metadata/root-doc alignment.

## Milestone 1 — External-onboarding docs stop teaching the wrong binary

Exit criteria:

- first-contact docs use installed/public `fretboard` spelling for public workflows,
- repo-maintainer docs explicitly keep `cargo run -p fretboard-dev -- ...`,
- and command examples no longer blur repo-local convenience with installed public product shape.

## Milestone 2 — Public `dev` contract defined

Exit criteria:

- the target public `dev native` / `dev web` argument model is written down,
- repo-only demo/cookbook shortcuts are explicitly separated,
- and follow-on implementation work can start without re-opening the taxonomy question.

## Milestone 3 — Public diagnostics core defined

Exit criteria:

- the current diagnostics tree is partitioned into public core vs repo extensions,
- the publish/dependency decision for that core is explicit,
- and docs can name the first-wave public diagnostics story without pointing at repo script
  catalogs.

## Milestone 4 — Deferred surfaces resolved

Exit criteria:

- hotpatch has an explicit public posture (`dev` submode or repo-only),
- `theme import-vscode` has an explicit public posture,
- and ADR/doc updates can present one coherent public CLI story.

## Done condition

This lane is done when:

- the public `fretboard` product story is explicit,
- repo-only `fretboard-dev` ownership is explicit,
- and remaining implementation work can be split into narrow code-focused follow-ons instead of
  further product-taxonomy debate.
