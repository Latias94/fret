# M2 Panel-Resize Gate Adoption — 2026-04-11

Status: accepted execution note

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `M1_CONTRACT_FREEZE_2026-04-11.md`
- `M2_CONSUMER_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M2_PANEL_RESIZE_GATE_PROMOTION_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/EVIDENCE_AND_GATES.md`

## Question

If this lane is not ready to promote a reusable public editor-rail surface yet, what should keep
the container-first requirement concrete in the meantime?

## Decision

Adopt the existing fixed-window panel-resize diagnostic proof from the closed adaptive lane into the
active gate set for this follow-on.

The current lane therefore treats the following three checks as the minimum proof bundle:

1. sidebar docs still pin `Sidebar` to the app-shell/device-shell lane,
2. `workspace_shell_demo` still proves the existing shell seam for a real editor rail,
3. the promoted panel-resize diagnostic still proves container-first adaptation under a fixed
   window.

## Why this is the right next slice

`M2_CONSUMER_AUDIT_2026-04-11.md` concludes that the repo still has only one real shell-mounted
editor-rail consumer.

That means the next best move is not premature API extraction.
It is to make the container-aware proof more central to this lane so any later extraction still has
to justify itself against a real panel-resize gate.

## Adopted gate

```bash
cargo run -p fretboard -- diag run tools/diag-scripts/container-queries-docking-panel-resize.json --dir target/fret-diag/adaptive-panel-resize-promote --session-auto --pack --include-screenshots --launch target/release/container_queries_docking_demo
```

Inherited evidence anchor:

- `docs/workstreams/adaptive-layout-contract-closure-v1/M2_PANEL_RESIZE_GATE_PROMOTION_2026-04-10.md`
- `target/fret-diag/adaptive-panel-resize-promote/sessions/1775822919781-88694`

## Consequence for this lane

The lane now has an active proof bundle for:

- app-shell boundary,
- shell seam,
- and container-first adaptive proof.

What remains open after this note is narrower:

1. whether a second real shell-mounted rail consumer exists,
2. and if not, whether the next slice should create one app-local consumer or just keep the
   current threshold frozen until evidence appears.
