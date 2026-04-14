# Executor-backed Mutation Surface v1 — Evidence & Gates

Goal: keep the mutation-surface lane tied to one real app probe, one shared executor substrate,
and one explicit docs/gate package instead of reopening broad state-surface debates by feel.

## Evidence anchors (current)

- `docs/workstreams/executor-backed-mutation-surface-v1/DESIGN.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/M0_BASELINE_AUDIT_2026-04-14.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/M1_CONTRACT_FREEZE_2026-04-14.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/TODO.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/MILESTONES.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/WORKSTREAM.json`
- `docs/audits/postman-like-api-client-first-contact.md`
- `docs/integrating-sqlite-and-sqlx.md`
- `docs/integrating-tokio-and-reqwest.md`
- `docs/crate-usage-guide.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`
- `ecosystem/fret-executor/src/lib.rs`
- `ecosystem/fret-executor/Cargo.toml`
- `ecosystem/fret-mutation/src/lib.rs`
- `ecosystem/fret-mutation/Cargo.toml`
- `ecosystem/fret-query/src/lib.rs`
- `ecosystem/fret-query/Cargo.toml`
- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret-genui-core/src/executor.rs`
- `ecosystem/fret-ui-shadcn/src/sonner.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `apps/fret-examples/Cargo.toml`
- `apps/fret-examples/src/lib.rs`
- `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-baseline.json`
- `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`

## First-open repro surfaces

1. API workbench consumer probe
   - `cargo run -p fret-demo --bin api_workbench_lite_demo`
2. Current shell + response diag proof
   - `cargo run -p fretboard -- diag run tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json --dir target/fret-diag-api-workbench-lite --session-auto --exit-after-run --launch cargo run -p fret-demo --bin api_workbench_lite_demo`
3. Current executor substrate
   - `cargo nextest run -p fret-executor`
4. Current mutation surface floor
   - `cargo nextest run -p fret-mutation --features ui`

## Current focused gates

### API workbench compile gate

- `cargo check -p fret-demo --bin api_workbench_lite_demo`

This gate currently proves:

- the first real consumer probe still builds,
- the repo still carries the current pressure surface,
- and future mutation-surface changes can be proven against a non-Todo example.

### API workbench diag gate

- `cargo run -p fretboard -- diag run tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json --dir target/fret-diag-api-workbench-lite --session-auto --exit-after-run --launch cargo run -p fret-demo --bin api_workbench_lite_demo`

This gate currently proves:

- the shell mounts,
- one send action leads to a reviewable terminal response state,
- and the repo has an artifact-producing proof surface for future submit-lifecycle regressions.

Latest passing evidence (2026-04-14):

- session dir:
  `target/fret-diag-api-workbench-lite-mutation/sessions/1776164998268-90687/`
- layout sidecar:
  `target/fret-diag-api-workbench-lite-mutation/sessions/1776164998268-90687/1776165124166-api-workbench-lite.shell-and-response.layout/layout.taffy.v1.json`
- screenshot:
  `target/fret-diag-api-workbench-lite-mutation/sessions/1776164998268-90687/screenshots/1776165124185-api-workbench-lite.shell-and-response/window-4294967297-tick-99-frame-98.png`
- bundle:
  `target/fret-diag-api-workbench-lite-mutation/sessions/1776164998268-90687/1776165124305-api-workbench-lite.shell-and-response/`

Latest strengthened SQLite-history evidence (2026-04-14):

- session dir:
  `target/fret-diag-api-workbench-lite-sqlite-history/sessions/1776168169993-16022/`
- layout sidecar:
  `target/fret-diag-api-workbench-lite-sqlite-history/sessions/1776168169993-16022/1776168310987-api-workbench-lite.shell-and-response.layout/layout.taffy.v1.json`
- bundle:
  `target/fret-diag-api-workbench-lite-sqlite-history/sessions/1776168169993-16022/1776168311008-api-workbench-lite.shell-and-response/`
- note:
  the script reached `api-workbench-lite.history.row.1` on the SQLite query lane, but
  `capture_screenshot` timed out at step 8, so this run is evidence for the stronger dataflow
  proof rather than the canonical passing screenshot artifact.

Latest passing SQLite-history artifact proof (2026-04-14):

- session dir:
  `target/fret-diag-api-workbench-lite-sqlite-history-rerun/sessions/1776168778413-22114/`
- layout sidecar:
  `target/fret-diag-api-workbench-lite-sqlite-history-rerun/sessions/1776168778413-22114/1776168911870-api-workbench-lite.shell-and-response.layout/layout.taffy.v1.json`
- screenshot:
  `target/fret-diag-api-workbench-lite-sqlite-history-rerun/sessions/1776168778413-22114/screenshots/1776168911906-api-workbench-lite.shell-and-response/window-4294967297-tick-285-frame-285.png`
- bundle:
  `target/fret-diag-api-workbench-lite-sqlite-history-rerun/sessions/1776168778413-22114/1776168912029-api-workbench-lite.shell-and-response/`
- note:
  a hot rerun of the same script passed without code changes, which points to diag timing noise in
  the first run rather than a contract or SQLite dataflow failure in the demo itself.

### Executor substrate floor

- `cargo nextest run -p fret-executor`

This gate currently proves:

- inbox delivery, wake behavior, cancellation-on-drop, and future-to-inbox bridging remain sound,
- which is the current mechanism substrate this lane inherits.

### Mutation surface floor

- `cargo nextest run -p fret-mutation --features ui`

This gate currently proves:

- the shared mutation state machine compiles and runs independently of the `fret` facade,
- send completions still cross the inbox boundary and materialize into success state,
- and cancellation still returns running mutations to idle without depending on query semantics.

### Docs source-policy gate

- `cargo nextest run -p fret docs_lock_query_reads_vs_mutation_submit_story`

This gate currently proves:

- the tokio/reqwest guide stays on observed-query semantics,
- the SQLite/SQLx guide keeps `cx.data().mutation_async(...)` + `handle.submit(...)` as the
  default explicit write path,
- and the crate-usage guide still names `fret-mutation` rather than falling back to raw
  executor-only teaching on the first-contact lane.

### API workbench SQLite proof source-policy gate

- `cargo nextest run -p fret-examples api_workbench_lite_demo_uses_query_for_sqlite_reads_and_mutation_for_explicit_submit`

This gate currently proves:

- the same Postman-like consumer probe now uses `query_async(...)` for SQLite history reads,
- explicit SQLite writes stay on `mutation_async(...)`,
- and the example no longer relies on local-only history bookkeeping for the first visible request
  history surface.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/executor-backed-mutation-surface-v1/WORKSTREAM.json > /dev/null`

## Missing gates before closure

Before claiming this lane is closed, add:

- at least one mutation-specific focused test package.
