---
title: Diag Script Library Modularization (Taxonomy + Suites)
status: draft
date: 2026-02-26
scope: diagnostics, scripted tests, UX, fearless-refactor
---

# Diag script library modularization (taxonomy + suites)

This doc records the **v1 decisions** for scaling the script library under `tools/diag-scripts/` without turning every
refactor into a repo-wide rewrite.

## Decisions (v1)

### D1: Built-in suites are curated directory inputs

Built-in `fretboard-dev diag suite <name>` suites are defined by curated directory inputs under:

- `tools/diag-scripts/suites/<suite-name>/`

In-tree suites are expressed via a single `suite.json` manifest (tooling-only) that lists canonical script paths.
Tooling expands suite manifests before pushing scripts, so suite membership does not reach the runtime contract surface.

### D2: Script paths become taxonomy-based (migrate with redirects)

The canonical script library is expected to move from a flat directory to a subfolder taxonomy (product area + intent).

Taxonomy rules (v1) live in:

- `tools/diag-scripts/migrate-script-library.py` (`categorize_script`)

Migration policy:

- Use redirects (`--write-redirects`) so old paths remain valid for:
  - docs / ADR evidence anchors,
  - examples,
  - suites (suite stubs can redirect to an old path which redirects again to the new location).

### D3: Suite strategy (today vs long-term)

Today:

- suites are directory-driven (curated stubs + deterministic `**/*.json` expansion),
- ad-hoc runs should prefer `--script-dir` / `--glob` inputs.
- `diag run` accepts either an explicit script path or a promoted `script_id` (resolved via `tools/diag-scripts/index.json`).
- `diag list scripts` prints `script_id -> path` for promoted scripts (same registry; intended for discoverability).
- `diag perf` suite expansion selects scripts by `suite_memberships` in the promoted registry, so perf suite naming stays
  stable as scripts move (as long as suite memberships are maintained via suite manifests).

Long-term:

- a minimal, generated registry exists at `tools/diag-scripts/index.json` (v1 scope: scripts reachable from in-tree
  suites + `_prelude`),
- in the future, prefer making suite membership tag-driven via the registry so it stays stable even as filenames evolve,
- keep directory/glob inputs as an escape hatch.

## Migration runbook (fearless refactor)

1) Dry-run and review a plan:

- `python tools/diag-scripts/migrate-script-library.py`
  - writes `.fret/diag-script-library-migration.plan.json` by default
  - for small batches, use filters like:
    - `--include-category ui_gallery.select`
    - `--include-prefix ui-gallery-select-`
    - `--include-name-glob ui-gallery-virtual-list-*.json`
    - `--exclude-category ui_gallery.misc`
    - `--limit 15`
  - consider using `--plan-out .fret/diag-script-library-migration.<label>.plan.json` to keep batch plans separate
  - when shrinking an intermediate bucket folder (e.g. `tools/diag-scripts/ui-gallery/misc/`), pass:
    - `--scan-dir tools/diag-scripts/ui-gallery/misc`
  - redirect stubs (`kind: script_redirect`) are ignored by the planner (only canonical scripts are migrated)

2) Apply moves with redirects (recommended):

- `python tools/diag-scripts/migrate-script-library.py --apply --write-redirects`
  - Suite manifests (`tools/diag-scripts/suites/**/suite.json`) are rewritten to point at the canonical (post-move)
    paths.

Tip (maintenance):

- After merging/pulling `main`, newly added scripts may land back in the flat `tools/diag-scripts/` root.
  Prefer re-running the migrator on a narrow filter (for example `--include-prefix ui-gallery-`) to keep the taxonomy
  invariant and avoid reintroducing “misc buckets”.
  - Example (batch-migrate a small set of newly added screenshot scripts):
    - `python tools/diag-scripts/migrate-script-library.py --apply --write-redirects --include-name-glob "*zinc-dark.json"`

3) Validate closures:

- Tooling-side health check (bounded, read-only; no Python required):
  - `cargo run -p fretboard-dev -- diag doctor scripts`

- Taxonomy drift check (for already-migrated areas):
  - `python tools/diag-scripts/migrate-script-library.py --check-root`
  - (optional) for incremental adoption, scope it:
    - `python tools/diag-scripts/migrate-script-library.py --check-root --include-prefix ui-gallery-`

- Suites still run (redirect chains are supported):
  - `cargo run -p fretboard-dev -- diag suite ui-gallery-select --launch -- cargo run -p fret-ui-gallery --release`
- Scriptgen closure still holds:
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-select`
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-combobox`
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-text-ime`
- Registry stays in sync:
  - `cargo run -p fretboard-dev -- diag registry check`

4) Normalize scripts (stable diffs):

- `cargo run -p fretboard-dev -- diag script normalize tools/diag-scripts/**.json --check`
- (optional) `--write` after reviewing diffs

## Non-goals (v1)

- Moving every script immediately (“big bang”).
- Forcing a registry before the taxonomy settles.

## Where to put new scripts (v1 guidance)

Goal: keep `tools/diag-scripts/` navigable as the library grows, and avoid dumping hundreds of unrelated scripts into a
single folder.

If in doubt, follow the filename prefix rules in `tools/diag-scripts/migrate-script-library.py` (`categorize_script`)
(the migrator is the source of truth for the current taxonomy).

### UI Gallery

Prefer placing UI Gallery scripts under `tools/diag-scripts/ui-gallery/<bucket>/` where `<bucket>` matches the component
or behavior area:

- `tools/diag-scripts/ui-gallery/select/` (prefix: `ui-gallery-select-`)
- `tools/diag-scripts/ui-gallery/combobox/` (prefix: `ui-gallery-combobox-`)
- `tools/diag-scripts/ui-gallery/overlay/` (prefix: `ui-gallery-…dialog|popover|tooltip|sheet|modal…`)
- `tools/diag-scripts/ui-gallery/perf/` (suffix: `-steady.json`, or tokens like `perf`, `resize`, `torture`)
- `tools/diag-scripts/ui-gallery/diag/` (prefix: `ui-gallery-diag-`, `ui-gallery-view-cache-`)
- Everything else (temporary compatibility only): `tools/diag-scripts/ui-gallery/misc/`

### Docking

Prefer placing docking scripts under `tools/diag-scripts/docking/<bucket>/`:

- `tools/diag-scripts/docking/arbitration/` (prefix: `docking-arbitration-`)
- `tools/diag-scripts/docking/motion-pilot/` (prefix: `docking-motion-pilot-`)
- `tools/diag-scripts/docking/demo/` (prefix: `docking-demo-`)
- `tools/diag-scripts/docking/container-queries/` (prefix: `container-queries-docking-`)

### Tooling / demos

Prefer placing non-UI-gallery scripts under `tools/diag-scripts/tooling/` (or the more specific top-level buckets when
they exist):

- Prelude/prewarm suite helpers: `tools/diag-scripts/_prelude/` (prefix: `tooling-suite-prelude-`, `tooling-suite-prewarm-`)
- `tools/diag-scripts/tooling/todo/` (prefix: `todo-`)
- `tools/diag-scripts/router/query-demo/` (prefix: `router-query-demo-`)
- `tools/diag-scripts/workspace/shell-demo/` (prefix: `workspace-shell-demo-`)
- `tools/diag-scripts/viewport/embedded-demo/` (prefix: `embedded-viewport-demo-`)

## Guardrails

After adding/moving scripts, keep drift visible and bounded:

- `cargo run -p fretboard-dev -- diag doctor scripts`
- `python tools/diag-scripts/migrate-script-library.py --check-root`
