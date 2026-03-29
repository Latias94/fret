# Fretboard CLI Fearless Refactor v1 — Milestones

Status: Closeout-ready
Last updated: 2026-03-26

Tracking doc: `docs/workstreams/fretboard-cli-fearless-refactor-v1/README.md`

## M0 — Scope and break policy

Outcome:

- The repo has one explicit statement that the top-level `fretboard` CLI is being simplified as a
  pre-release hard reset, not a compatibility lane.

Deliverables:

- A dedicated workstream folder.
- A written no-compatibility policy for repo-owned command families.
- A recorded target ownership model for top-level shell, cutover, and family-local contracts.

Exit criteria:

- A maintainer can answer which top-level surfaces are intentionally allowed to break and why.

## M1 — Modular shell ownership

Outcome:

- `apps/fretboard` has one stable modular shell shape for future command work.

Deliverables:

- A typed top-level command tree.
- A dedicated cutover dispatch layer.
- A family-local contract pattern that avoids regressing into one parser blob.

Exit criteria:

- New command-family work has an obvious place to land without widening the shell into another god
  module.

## M2 — Family migration

Outcome:

- Repo-owned command families are parsed through typed contracts with modular execution ownership.

Current progress:

- `assets`, `dev`, `hotpatch`, `config`, and `theme` are already migrated.
- `diag` remains delegated to its canonical typed contract in `crates/fret-diag`.
- `new` is now migrated to a typed contract shape.
- `init` has been deleted instead of preserved as a compatibility alias.

Deliverables:

- `new` migrated to a typed contract shape.
- `init` removed instead of preserved as a compatibility alias.
- Family-local tests covering representative valid and invalid invocations.

Exit criteria:

- No repo-owned top-level command family still depends on hand-written argv loops.

## M3 — Root help and docs closure

Outcome:

- The root shell teaches only the shipped command surface and no longer drifts from executable
  contracts.

Deliverables:

- Root help reduced to contract-driven output or a narrow curated overlay.
- Updated repo-owned docs/snippets for any changed `new` surface.
- Help/parser gates for the final scaffold entrypoints.

Exit criteria:

- Reviewers can trust `fretboard --help` and first-party docs without cross-checking a second,
  hand-maintained syntax source.

## M4 — Closeout

Outcome:

- `fretboard` reaches the intended pre-release CLI structure with no retained compatibility debt.

Deliverables:

- A short closeout note.
- Final smoke commands and evidence anchors.
- A small remaining-followups list only if new requirements appear, not because parser debt was
  deferred.

Exit criteria:

- The top-level CLI is typed, modular, and free of compatibility-only aliases.
