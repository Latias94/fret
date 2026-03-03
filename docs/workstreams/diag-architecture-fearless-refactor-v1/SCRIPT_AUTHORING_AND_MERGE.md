# Diag scripts: authoring and merge-friendly practices

This note explains why `tools/diag-scripts/**/*.json` often cause merge conflicts and what to do about it.

The goal is to keep scripted diagnostics:

- **portable** (plain JSON committed to the repo),
- **reviewable** (small, focused diffs),
- **merge-friendly** (avoid editing the same giant file in multiple branches),
- **stable** (deterministic execution + deterministic artifacts).

## 1) Prefer many small scripts over one giant sweep

Merge conflicts scale with file size and "hot" shared files. The easiest win is structural:

- One behavior per script.
- Keep scripts short (rule of thumb: ~50–200 steps; split beyond that).
- Compose large coverage using **suite membership**, not a single mega-script.

Suite membership is curated via suite manifests:

- Add/modify membership under `tools/diag-scripts/suites/<suite-name>/suite.json` using `kind=diag_script_suite_manifest`.
- Keep canonical scripts under `tools/diag-scripts/**/`.

This is low-noise in the tree, but can increase merge conflicts if many people edit the same suite concurrently.

Mitigations:

- Keep suites small and focused (split large suites).
- Prefer adding new scenarios as new scripts and then appending their path to the suite manifest (keep one change per line).
- When conflicts happen, resolve by re-sorting/normalizing the manifest and regenerating the promoted index (see below).

## 2) Normalize script formatting before committing

Formatting churn causes conflicts even when semantics did not change. Use the built-in tooling formatter:

- Normalize + rewrite:
  - `cargo run -p fretboard -- diag script normalize tools/diag-scripts/ui-gallery/perf/foo.json --write`
- Or normalize all changed scripts in your working tree (and refresh `index.json`):
  - `powershell -ExecutionPolicy Bypass -File tools/diag_scripts_refresh.ps1`
- Normalize + check (CI-friendly for a local gate):
  - `cargo run -p fretboard -- diag script normalize tools/diag-scripts/ui-gallery/perf/foo.json --check`

Notes:

- Normalization is **JSON-canonicalization + pretty printing** (stable diffs).
- This is not currently enforced repo-wide; apply it to scripts you touch.

## 3) Use lint/validate to keep scripts robust (and reduce churn)

Schema validation (parse-only):

- `cargo run -p fretboard -- diag script validate tools/diag-scripts`

Lint (capability inference + hygiene; encourages consistent `meta.required_capabilities`):

- `cargo run -p fretboard -- diag script lint tools/diag-scripts`

If you still have legacy scripts:

- `cargo run -p fretboard -- diag script upgrade <script.json> --write`

## 4) Avoid hand-merging generated artifacts

Some files are derived and should not be manually merged:

- `tools/diag-scripts/index.json` is generated.

If it conflicts:

- resolve by rerunning:
  - `cargo run -p fretboard -- diag registry write`

Then re-run:

- `cargo run -p fretboard -- diag registry check`

## 5) For large matrices, generate scripts (do not edit by hand)

When a scenario needs loops, branching, or a large cartesian product:

- Prefer a small generator (Rust or Python) that emits multiple scripts.
- Keep each output script focused and deterministic.
- Put the generator next to the outputs (or in a dedicated `tools/` folder) and treat it as the source of truth.

`docs/ui-diagnostics-and-scripted-tests.md` also documents `fret-diag-scriptgen` for typed templates.

## 6) Practical “conflict reducers” inside a script

These patterns reduce edit churn:

- Prefer `test_id` selectors (stable against wording/localization).
- Keep navigation boilerplate consistent (same wait-until patterns, same window sizing step).
- Use `reset_diagnostics` before measurement-heavy sequences.
- Prefer `wait_until` predicates over adding arbitrary delays (fewer "tune the number" commits).

## 7) Future work (tooling-only composition)

If we still see frequent conflicts in big sweeps, the next step is a tooling-only composition layer:

- A “script pack” format (directory + manifest) that expands to a v2 script before pushing to the runtime.
- This avoids changing the runtime contract while enabling split ownership (one file per page/section).

If we pursue this, it should be proposed via a small design note/ADR first.

## Appendix: bulk migration tool

To migrate suite directories from legacy redirect stubs to suite manifests:

- `python tools/migrate_diag_suites_to_manifest.py --apply`
