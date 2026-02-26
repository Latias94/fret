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

Built-in `fretboard diag suite <name>` suites are defined by curated directory inputs under:

- `tools/diag-scripts/suites/<suite-name>/`

Each entry is a small `script_redirect` stub pointing at a canonical script path. Tooling resolves redirects before
executing/pushing scripts, so redirects do not reach the runtime contract surface.

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

Long-term:

- prefer a registry (`tools/diag-scripts/index.json` or similar) so suite membership can be tag-driven and stable even as
  filenames evolve,
- keep directory/glob inputs as an escape hatch.

## Migration runbook (fearless refactor)

1) Dry-run and review a plan:

- `python tools/diag-scripts/migrate-script-library.py`
  - writes `.fret/diag-script-library-migration.plan.json` by default
  - for small batches, use filters like `--include-category ui_gallery.select` and `--limit 15`

2) Apply moves with redirects (recommended):

- `python tools/diag-scripts/migrate-script-library.py --apply --write-redirects`

3) Validate closures:

- Suites still run (redirect chains are supported):
  - `cargo run -p fretboard -- diag suite ui-gallery-select --launch -- cargo run -p fret-ui-gallery --release`
- Scriptgen closure still holds:
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-select`
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-combobox`
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-text-ime`

4) Normalize scripts (stable diffs):

- `cargo run -p fretboard -- diag script normalize tools/diag-scripts/**.json --check`
- (optional) `--write` after reviewing diffs

## Non-goals (v1)

- Moving every script immediately (“big bang”).
- Forcing a registry before the taxonomy settles.
