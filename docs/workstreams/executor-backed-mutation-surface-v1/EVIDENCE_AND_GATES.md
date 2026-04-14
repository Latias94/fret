# Executor-backed Mutation Surface v1 — Evidence & Gates

Goal: keep the mutation-surface lane tied to one real app probe, one shared executor substrate,
and one explicit docs/gate package instead of reopening broad state-surface debates by feel.

## Evidence anchors (current)

- `docs/workstreams/executor-backed-mutation-surface-v1/DESIGN.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/M0_BASELINE_AUDIT_2026-04-14.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/TODO.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/MILESTONES.md`
- `docs/workstreams/executor-backed-mutation-surface-v1/WORKSTREAM.json`
- `docs/audits/postman-like-api-client-first-contact.md`
- `docs/integrating-sqlite-and-sqlx.md`
- `docs/integrating-tokio-and-reqwest.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`
- `ecosystem/fret-executor/src/lib.rs`
- `ecosystem/fret-query/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret-genui-core/src/executor.rs`
- `ecosystem/fret-ui-shadcn/src/sonner.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-baseline.json`
- `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`

## First-open repro surfaces

1. API workbench consumer probe
   - `cargo run -p fret-demo --bin api_workbench_lite_demo`
2. Current shell + response diag proof
   - `cargo run -p fretboard -- diag run tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json --dir target/fret-diag-api-workbench-lite --session-auto --exit-after-run --launch cargo run -p fret-demo --bin api_workbench_lite_demo`
3. Current executor substrate
   - `cargo nextest run -p fret-executor`

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

### Executor substrate floor

- `cargo nextest run -p fret-executor`

This gate currently proves:

- inbox delivery, wake behavior, cancellation-on-drop, and future-to-inbox bridging remain sound,
- which is the current mechanism substrate this lane inherits.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/executor-backed-mutation-surface-v1/WORKSTREAM.json > /dev/null`

## Missing gates before closure

Before claiming this lane is closed, add:

- at least one mutation-specific focused test package,
- source-policy protection that first-contact examples do not drift back to `query_async(...)` for
  explicit submit flows,
- and one second real consumer beyond the API workbench.
