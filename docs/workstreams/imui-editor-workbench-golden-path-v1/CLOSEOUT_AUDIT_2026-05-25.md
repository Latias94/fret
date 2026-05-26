# IMUI Editor Workbench Golden Path v1 - Closeout Audit - 2026-05-25

Status: closed
Last updated: 2026-05-25

## Objective

Close the canonical editor workbench lane after proving:

1. the product-facing workbench route exists and is discoverable,
2. the route mounts a real editor workflow instead of forwarding to a proof demo,
3. Demo/Metrics/Debug and docking handoffs have explicit owner lanes,
4. proof-led helper/API candidates were accepted or deferred from evidence, and
5. follow-on implementation lanes have their own gates.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| Canonical workbench route exists | `apps/fret-examples/src/imui_editor_workbench_demo.rs`, `apps/fret-demo/src/bin/imui_editor_workbench_demo.rs` |
| Route mounts editor-notes workflow directly | `apps/fret-examples/src/editor_notes_demo.rs`, `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs` |
| Product discovery promotes the route | `docs/examples/README.md`, `apps/fret-cookbook/README.md`, `apps/fretboard/src/demos.rs` |
| Demo/Metrics/Debug productization split to owner lane | `docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json` |
| Docking/runner hand-feel stayed in docking owner lane | `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json` |
| ListBox, plot adapter, style/theme picker, and table owner splits have narrow lanes | `docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json`, `docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json`, `docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json`, `docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json`, `docs/workstreams/imui-table-body-owner-split-v1/WORKSTREAM.json` |
| Focused workbench gates passed | `docs/workstreams/imui-editor-workbench-golden-path-v1/EVIDENCE_AND_GATES.md` |

## Residual Boundaries

- Real Wayland compositor acceptance remains open in
  `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`; this lane must not claim it.
- Broad porting sugar remains deferred until at least two product routes need the same shorthand.
- Future workbench product depth should start as a new follow-on or in the relevant owner lane, not
  by reopening this golden-path lane.

## Outcome

The workbench lane is closed as the product-facing IMUI golden path. Additive workflow depth,
DevTools execution controls, perf attribution, runner/backend work, or public helper growth should
start from fresh proof pressure and keep the same owner boundaries.
