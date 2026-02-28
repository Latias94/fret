---
title: Diag Script Suites (Redirect Stubs)
status: living
scope: diagnostics, scripted tests, suite membership
---

# Diag script suites (redirect stubs)

Built-in `fretboard diag suite <name>` suites are defined as curated directory inputs under:

- `tools/diag-scripts/suites/<suite-name>/`

Discoverability:

- List suites: `cargo run -p fretboard -- diag list suites`

Each JSON file in a suite directory is a small `script_redirect` stub that points at the canonical script path under
`tools/diag-scripts/` (or elsewhere in the repo).

Why:

- Keep suite membership curated (not “run everything under tools/diag-scripts”).
- Make script library refactors fearless: path moves can leave behind redirects, and suites can be updated by editing stubs.
- Avoid Rust-side hard-coded lists of script paths.

Notes:

- Tooling resolves redirects before pushing scripts to the runtime; redirects are not part of the runtime contract.
- Suite execution order is deterministic and derived from the expanded input paths (lexicographic path ordering).
- `fretboard diag suite <name>` prefers `tools/diag-scripts/suites/<name>/` when it exists, so adding a new suite does not
  require Rust-side edits (suite-specific env defaults can still live in tooling or script `meta.env_defaults`).
- `diag perf` suite membership is also expressed via suite directories (typically `tools/diag-scripts/suites/perf-*/`),
  and is materialized into the promoted registry as `suite_memberships`.
- A minimal, generated registry exists at `tools/diag-scripts/index.json` (scope: suite-reachable scripts + `_prelude`)
  and is validated in CI via `python tools/check_diag_scripts_registry.py`.
