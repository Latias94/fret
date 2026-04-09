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

Current state:

- reached on 2026-04-09 through the first-contact docs sweep in:
  - `docs/README.md`
  - `docs/setup.md`
  - `docs/first-hour.md`
  - `docs/examples/README.md`
  - `docs/examples/todo-app-golden-path.md`

## Milestone 2 — Public `dev` contract defined

Exit criteria:

- the target public `dev native` / `dev web` argument model is written down,
- repo-only demo/cookbook shortcuts are explicitly separated,
- and follow-on implementation work can start without re-opening the taxonomy question.

Current state:

- target interface frozen on 2026-04-09 in
  `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`
- repo convenience separation (`--demo`, chooser, gallery/cookbook shortcuts) is now an execution
  item rather than an open taxonomy debate

## Milestone 3 — Public diagnostics core defined

Exit criteria:

- the current diagnostics tree is partitioned into public core vs repo extensions,
- the publish/dependency decision for that core is explicit,
- and docs can name the first-wave public diagnostics story without pointing at repo script
  catalogs.

Current state:

- reached on 2026-04-09 with the diagnostics target-state freeze in
  `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
- public diagnostics remain a `fretboard diag` product surface,
  while `crates/fret-diag` may still need publication as an implementation prerequisite
- repo-only diagnostics ownership is now explicit for suite/registry/campaign and related
  mono-repo inventory helpers

## Milestone 4 — Deferred surfaces resolved

Exit criteria:

- hotpatch has an explicit public posture (`dev` submode or repo-only),
- `theme import-vscode` has an explicit public posture,
- and ADR/doc updates can present one coherent public CLI story.

Current state:

- hotpatch posture was resolved on 2026-04-09 in
  `docs/workstreams/fretboard-public-app-author-surface-v1/HOTPATCH_TARGET_INTERFACE_STATE.md`
- reached on 2026-04-09 once `theme import-vscode` was explicitly kept off public `fretboard` and
  assigned a future dedicated-package posture in
  `docs/workstreams/fretboard-public-app-author-surface-v1/THEME_TARGET_INTERFACE_STATE.md`

## Done condition

This lane is done when:

- the public `fretboard` product story is explicit,
- repo-only `fretboard-dev` ownership is explicit,
- and remaining implementation work can be split into narrow code-focused follow-ons instead of
  further product-taxonomy debate.
