# Workstream State v1

Status: Active shared convention
Last updated: 2026-04-02

This note defines a very small machine-readable state file for dedicated workstream directories:

- `docs/workstreams/<slug>/WORKSTREAM.json`

The goal is to help humans and agents answer four first-open questions quickly:

1. Is this lane active, maintenance-only, closed, or historical?
2. Which docs are authoritative right now?
3. What is the first repro/gate surface to run or inspect?
4. Should work continue in this lane, or start as a narrower follow-on?

This file is intentionally not a planner, transcript, or duplicate TODO system.
It should summarize lane state, not replace the existing markdown docs.

## Positioning

Use `WORKSTREAM.json` as:

- a first-open index for an existing lane,
- a handoff aid for humans and agents,
- and a lightweight state contract that CI can validate.

Do not use it as:

- a second task tracker,
- a dump of every command ever run,
- or a place to invent state that the markdown docs do not support.

Repo-wide stance still comes first:

- `docs/roadmap.md`
- `docs/workstreams/README.md`
- `docs/todo-tracker.md`

After that, if a lane has `WORKSTREAM.json`, open it before diving into older notes.

## File location

Only place `WORKSTREAM.json` inside a dedicated lane directory:

- `docs/workstreams/<slug>/WORKSTREAM.json`

Do not place it at `docs/workstreams/` top level.

## Path rule

All paths inside `WORKSTREAM.json` are repo-root relative.

Examples:

- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `apps/fretboard/src/scaffold/templates.rs`
- `tools/diag-campaigns/ui-gallery-smoke.json`

## v1 schema

Required top-level fields:

- `schema_version`
  - integer
  - v1 value: `1`
- `slug`
  - string
  - must match the containing directory name
- `title`
  - human-readable lane title
- `status`
  - one of:
    - `active`
    - `maintenance`
    - `closed`
    - `historical`
- `updated`
  - `YYYY-MM-DD`
- `scope_kind`
  - one of:
    - `execution`
    - `closeout`
    - `audit`
    - `reference`
- `problem`
  - one short paragraph describing the invariant this lane owns
- `authoritative_docs`
  - ordered list of objects:
    - `role`
    - `path`
- `repro`
  - object:
    - `summary`
    - `commands` (non-empty list of strings)
- `gates`
  - non-empty list of objects:
    - `name`
    - `command`
- `evidence`
  - non-empty list of repo-root-relative paths
- `adr_refs`
  - list of repo-root-relative ADR or alignment paths
  - may be empty
- `continue_policy`
  - object:
    - `default_action`
    - `notes`

Optional top-level fields:

- `follow_on_of`
  - slug of the prior lane when this lane is an explicit follow-on

## `authoritative_docs` roles

Use the smallest role set that keeps the lane easy to reopen:

- `positioning`
  - lane overview, usually `README.md` or `<slug>.md`
- `design`
  - scope and intended target surface
- `target`
  - target interface or target shipped state
- `status`
  - current state or current priorities note
- `next`
  - explicit near-term next-priority note
- `execution`
  - `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`
- `closeout`
  - closeout audit or final status note
- `reference`
  - high-value supporting note that frequently matters during reopen

Keep the list ordered by first-open value.

## `continue_policy.default_action`

Allowed values:

- `continue`
  - keep working in this lane
- `start_follow_on`
  - do not reopen this lane broadly; create a narrower follow-on if new work appears
- `stay_closed`
  - treat the lane as closed/history unless a future explicit state change rewrites this file

Recommended pairing:

- `active` or `maintenance` -> `continue`
- `closed` or `historical` -> `start_follow_on` or `stay_closed`

## Precedence rule

`WORKSTREAM.json` is a first-open summary, not the final authority.

If it conflicts with:

- a closeout audit,
- an explicit top-of-file status note,
- or repo-wide roadmap/workstream-index stance,

the markdown authority wins and `WORKSTREAM.json` should be updated.

## Minimal example

```json
{
  "schema_version": 1,
  "slug": "example-workstream-v1",
  "title": "Example Workstream v1",
  "status": "active",
  "updated": "2026-04-02",
  "scope_kind": "execution",
  "problem": "Keep one short statement of the invariant this lane owns.",
  "authoritative_docs": [
    {
      "role": "positioning",
      "path": "docs/workstreams/example-workstream-v1/README.md"
    },
    {
      "role": "execution",
      "path": "docs/workstreams/example-workstream-v1/TODO.md"
    },
    {
      "role": "execution",
      "path": "docs/workstreams/example-workstream-v1/MILESTONES.md"
    }
  ],
  "repro": {
    "summary": "Smallest first-open command for this lane.",
    "commands": [
      "cargo run -p fretboard -- diag doctor campaigns"
    ]
  },
  "gates": [
    {
      "name": "campaign-doctor",
      "command": "cargo run -p fretboard -- diag doctor campaigns"
    }
  ],
  "evidence": [
    "docs/workstreams/example-workstream-v1/README.md"
  ],
  "adr_refs": [],
  "continue_policy": {
    "default_action": "continue",
    "notes": "Continue this lane until the scope is explicitly closed."
  }
}
```

## Recommended usage

Create or refresh `WORKSTREAM.json` when:

- a dedicated lane is started,
- lane status changes,
- the authoritative first-open doc set changes,
- or the primary repro/gate surface changes.

Do not churn it for every tiny checklist edit.
