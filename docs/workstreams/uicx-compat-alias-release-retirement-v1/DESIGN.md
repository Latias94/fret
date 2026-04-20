# UiCx Compat Alias Release Retirement v1

Status: Closed historical design note
Last updated: 2026-04-20

Status note (2026-04-20): this document remains useful for the lane-opening rationale and the
release-facing owner split, but the shipped verdict now lives in
`CLOSEOUT_AUDIT_2026-04-20.md` and `WORKSTREAM.json`. Read the execution framing below as the
historical lane setup that led to full alias deletion.

Related:

- `M0_BASELINE_AUDIT_2026-04-19.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `release-plz.toml`

This workstream is a narrow release-facing follow-on to
`public-authoring-state-lanes-and-identity-fearless-refactor-v1`.

It does not reopen the broad render-authoring surface refactor.
That lane is already closed on the correct architectural outcome:

- first-party default teaching now uses `AppComponentCx<'a>`, `AppRenderCx<'a>`, and
  `AppRenderContext<'a>`,
- `UiCx<'a>` no longer carries default authoring meaning,
- and the remaining question is whether the deprecated compatibility alias family should stay for a
  defined release window or be removed later with explicit release evidence.

## Why this lane exists

The repo now has a clean default app-facing authoring story, but the publish-facing `fret` facade
still exposes a deprecated compatibility alias family:

- `UiCx<'a>`
- `UiCxActionsExt`
- `UiCxDataExt`
- hidden deprecated carrier aliases in `ecosystem/fret/src/view.rs`

Because `fret` is part of the published release graph and `release-plz.toml` enables
`semver_check = true`, deleting those names is no longer a purely internal refactor.
It is a release-policy decision that should be made explicitly.

## Must-be-true outcomes

1. The repo keeps one canonical teaching surface:
   `AppComponentCx<'a>`, `AppRenderCx<'a>`, and `AppRenderContext<'a>` remain the first-party
   answer regardless of the compatibility verdict.
2. The live `UiCx*` alias inventory is frozen and classified by visibility:
   explicit public compatibility exports vs `doc(hidden)` deprecated carriers vs historical docs.
3. The release window is explicit:
   the repo can say whether the aliases are retained for a defined migration window or removed in a
   defined release-facing slice.
4. Removal criteria are explicit:
   no alias is deleted merely because first-party code no longer uses it; deletion must name the
   downstream evidence, release note, and gate set that make the breakage acceptable.

## In scope

- Freeze the current `UiCx*` compatibility inventory on the published `fret` surface.
- Decide whether explicit public aliases and hidden deprecated carriers retire together or on
  different release windows.
- Define the migration note and deletion criteria for any future removal.
- Keep release-facing docs, source-policy gates, and tests aligned with the chosen verdict.

## Out of scope

- Reopening the broad public authoring surface or `LocalState<T>` refactor.
- Renaming the canonical app-facing helper surfaces again.
- Re-teaching `UiCx<'a>` on first-party examples, cookbook paths, or UI Gallery snippets.
- Blindly deleting compatibility aliases in this opener without a frozen release verdict.

## Owner split

### `ecosystem/fret`

Owns the actual compatibility exports and any future alias-removal implementation.

### `docs/workstreams/*` and repo entry docs

Own the release-facing decision record:

- why the broad lane stays closed,
- which aliases still exist,
- and what conditions must hold before deletion.

### Release tooling and source-policy gates

Own the proof that this is not merely a style cleanup:

- `release-plz.toml`
- `tools/gate_no_raw_app_context_in_default_teaching_snippets.py`
- targeted `fret` source-policy tests

## Target shipped state

The repo should be able to answer this question without reopening the old lane:

> What is the current release policy for the deprecated `UiCx` compatibility alias family?

The final answer may be either:

- retain the compatibility aliases for one explicit release window while keeping them deprecated and
  out of first-party teaching, or
- remove them in a release-facing slice once the repo has explicit downstream and semver evidence.

That answer is now recorded in `CLOSEOUT_AUDIT_2026-04-20.md`: the repo deleted the entire
`UiCx*` compatibility family instead of carrying it forward into another public release window.
