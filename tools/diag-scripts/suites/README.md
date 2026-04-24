---
title: Diag Script Suites (Redirect Stubs + Manifests)
status: living
scope: diagnostics, scripted tests, suite membership
---

# Diag script suites (suite manifests)

Built-in `fretboard-dev diag suite <name>` suites are defined as curated directory inputs under:

- `tools/diag-scripts/suites/<suite-name>/`

Discoverability:

- List suites: `cargo run -p fretboard-dev -- diag list suites`

Suite membership is expressed via a tooling-only suite manifest:

1) **Suite manifest** (default): a single `suite.json` (or `_suite.json`) file at the suite root with:
   - `kind: "diag_script_suite_manifest"`
   - `schema_version: 1`
   - `scripts: ["tools/diag-scripts/...", ...]`

Legacy format (still supported by tooling, but discouraged in-tree):

- **Redirect stubs**: each JSON file in a suite directory is a small `script_redirect` stub that points at a canonical
  script path. This was useful for merge-friendliness, but creates large file-count noise in the repo tree.

Why:

- Keep suite membership curated (not “run everything under tools/diag-scripts”).
- Make script library refactors fearless: path moves can leave behind redirects, and suites can be updated by editing stubs.
- Avoid Rust-side hard-coded lists of script paths.

Choosing a format:

- In this repo, prefer the **suite manifest** format by default.
- If you need stub-style merge behavior for a high-churn suite, consider:
  - splitting the suite into smaller suites to reduce concurrent edits, or
  - maintaining out-of-tree stub suites (tooling still supports them).

Notes:

- Tooling resolves redirects before pushing scripts to the runtime; redirects are not part of the runtime contract.
- Suite execution order is deterministic and derived from the expanded input paths (lexicographic path ordering).
- Nested suite manifests are valid when a broad suite needs named subsets, e.g.
  `tools/diag-scripts/suites/docking-arbitration/common/suite.json` and
  `tools/diag-scripts/suites/docking-arbitration/windows/suite.json`.
- `fretboard-dev diag suite <name>` prefers `tools/diag-scripts/suites/<name>/` when it exists, so adding a new suite does not
  require Rust-side edits (suite-specific env defaults can still live in tooling or script `meta.env_defaults`).
- `diag perf` suite membership is also expressed via suite directories (typically `tools/diag-scripts/suites/perf-*/`),
  and is materialized into the promoted registry as `suite_memberships`.
- A minimal, generated registry exists at `tools/diag-scripts/index.json` (scope: suite-reachable scripts + `_prelude`)
  and is validated in CI via `cargo run -p fretboard-dev -- diag registry check` (or the legacy Python validator
  `python tools/check_diag_scripts_registry.py`).

Editing a suite safely:

- Add a script to a suite: `python tools/diag_suite_edit.py add <suite> <script.json> --refresh-index`
- Remove a script: `python tools/diag_suite_edit.py remove <suite> <script.json> --refresh-index`
- Canonicalize ordering (sorted + de-duped): `python tools/diag_suite_edit.py fmt --suite <suite> --refresh-index`
