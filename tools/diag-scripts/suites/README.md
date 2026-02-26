---
title: Diag Script Suites (Redirect Stubs)
status: living
scope: diagnostics, scripted tests, suite membership
---

# Diag script suites (redirect stubs)

Built-in `fretboard diag suite <name>` suites are defined as curated directory inputs under:

- `tools/diag-scripts/suites/<suite-name>/`

Each JSON file in a suite directory is a small `script_redirect` stub that points at the canonical script path under
`tools/diag-scripts/` (or elsewhere in the repo).

Why:

- Keep suite membership curated (not “run everything under tools/diag-scripts”).
- Make script library refactors fearless: path moves can leave behind redirects, and suites can be updated by editing stubs.
- Avoid Rust-side hard-coded lists of script paths.

Notes:

- Tooling resolves redirects before pushing scripts to the runtime; redirects are not part of the runtime contract.
- Suite execution order is deterministic and derived from the expanded input paths (lexicographic path ordering).
