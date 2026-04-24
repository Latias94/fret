# ImUi Interaction Inspector v1 Evidence And Gates

Status: closed gate list
Last updated: 2026-04-24

## Smallest Repro

```bash
cargo run -p fret-demo --bin imui_interaction_showcase_demo
```

Use the pulse button, drag probe, menu, tabs, and context menu. The inspector should update with
the latest response flags while the timeline remains a short audit trail.

## Required Gates

```bash
cargo fmt --package fret-examples
cargo check -p fret-examples
cargo nextest run -p fret-examples imui_interaction_showcase --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/imui-interaction-inspector-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

Lane docs:

- `docs/workstreams/imui-interaction-inspector-v1/WORKSTREAM.json`
- `docs/workstreams/imui-interaction-inspector-v1/DESIGN.md`
- `docs/workstreams/imui-interaction-inspector-v1/TODO.md`
- `docs/workstreams/imui-interaction-inspector-v1/MILESTONES.md`
- `docs/workstreams/imui-interaction-inspector-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-interaction-inspector-v1/CLOSEOUT_AUDIT_2026-04-24.md`

Implementation anchors:

- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/lib.rs`

Reference lane anchors:

- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`

## Verified Gates

Passed on 2026-04-24:

```bash
cargo fmt --package fret-examples
cargo check -p fret-examples
cargo nextest run -p fret-examples imui_interaction_showcase --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/imui-interaction-inspector-v1/WORKSTREAM.json
git diff --check
```
