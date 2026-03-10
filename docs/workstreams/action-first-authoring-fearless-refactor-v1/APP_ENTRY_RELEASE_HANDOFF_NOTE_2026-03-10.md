# Action-First Authoring + View Runtime (Fearless Refactor v1) — `App::ui*` Release Handoff Note (2026-03-10)

Status: draft, release handoff
Last updated: 2026-03-10

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_RELEASE_EVIDENCE_TRACKER_2026-03-09.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- `docs/release/v0.1.0-release-checklist.md`
- `release-plz.toml`

---

## Purpose

This note tells the release operator exactly what to copy back into the workstream once the first
published deprecated `fret` release ships.

It exists to prevent the final `App::ui*` delete/quarantine decision from depending on memory or a
manual archaeology pass across release artifacts.

---

## When to use

Use this note only when a published `fret` release is being verified after `release-plz` runs.

Do not use it to justify early removal.
The policy window from `APP_ENTRY_POLICY_DECISION_DRAFT.md` still applies.

---

## What to record

After the first published deprecated `fret` release ships, update
`APP_ENTRY_RELEASE_EVIDENCE_TRACKER_2026-03-09.md` with these fields:

| Field | Required value |
| --- | --- |
| Released `fret` version | The exact crate version published to crates.io |
| Publish date | The actual publish date used for the downstream window audit |
| Release anchor | Release PR / GitHub Release / tag anchor used to review the publish event |
| Crates.io anchor | Public crate page for the published `fret` version |
| Docs anchor | Published docs location that shows the default `view::<V>()` / `view_with_hooks::<V>(...)` path |
| Deprecation proof | A short note stating that `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` are already deprecated in the published surface |

---

## Minimal evidence sources

Prefer these sources, in this order:

1. `release-plz` release PR / merge result
2. Git tag or GitHub Release created for that publish
3. crates.io page for `fret`
4. docs.rs page for `fret`

If one source is missing, do not guess.
Record the other anchors and leave the missing field explicitly open.

---

## Minimal backfill template

Add a short block like this under the tracker:

```md
- Published deprecated release recorded: `fret vX.Y.Z`
  - Publish date: YYYY-MM-DD
  - Release anchor: <link or identifier>
  - Crates.io anchor: <link>
  - Docs anchor: <link>
  - Verification: published docs still teach `view::<V>()` / `view_with_hooks::<V>(...)`, and
    `App::ui*` remains deprecated rather than restored as default path.
```

Keep the update short.
This is evidence capture, not a release retrospective.

---

## Completion rule

The handoff is complete when both are true:

1. `APP_ENTRY_RELEASE_EVIDENCE_TRACKER_2026-03-09.md` no longer shows the published-release row as
   open, and
2. the release evidence is specific enough that a later reviewer can confirm the compatibility
   window existed without re-running the release workflow.

Only after that, and only after **2026-06-09**, should the repo reopen
`APP_ENTRY_REMOVAL_PLAYBOOK.md`.
