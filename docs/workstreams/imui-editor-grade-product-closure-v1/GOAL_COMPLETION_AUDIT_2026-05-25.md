# ImUi Editor-Grade Product Closure Goal Completion Audit - 2026-05-25

Status: Implementation slices closed; full product-closure goal still has external owner-lane blockers.
Last updated: 2026-05-25

## What This Audit Refreshes

This refresh records the follow-on work completed after the 2026-05-15 audit:

- canonical editor workbench route closed,
- Demo/Metrics/Debug route/product action surface closed,
- ListBox container proof closed,
- optional plot adapter proof closed,
- style/theme preset picker proof and canonical workbench integration closed,
- private IMUI table header/body owner splits closed.

It does not replace the strict completion rule from the earlier audits: do not claim real
platform-specific hand-feel without real-host evidence.

## Completion Delta Since 2026-05-15

| Requirement | New evidence | Verdict |
| --- | --- | --- |
| Product-facing editor workbench route | `docs/workstreams/imui-editor-workbench-golden-path-v1/CLOSEOUT_AUDIT_2026-05-25.md` | Met for canonical route |
| Demo/Metrics/Debug discoverability and action surface | `docs/workstreams/imui-demo-metrics-debug-devtools-v1/CLOSEOUT_AUDIT_2026-05-25.md` | Met for CLI/GUI/MCP first-open route and copyable actions |
| ListBox-style container parity | `docs/workstreams/imui-list-box-container-proof-v1/CLOSEOUT_AUDIT_2026-05-25.md` | Met for narrow container semantics |
| Plot adapter parity | `docs/workstreams/imui-plot-adapter-proof-v1/CLOSEOUT_AUDIT_2026-05-25.md` | Met for optional declarative `fret-plot/imui` adapter |
| Style/theme editor first affordance | `docs/workstreams/imui-style-theme-editor-proof-v1/CLOSEOUT_AUDIT_2026-05-25.md` | Met for editor-owned preset picker, not broad `GetStyle`/`PushStyleVar` clone |
| `fret-ui-kit::imui` owner split pressure | `docs/workstreams/imui-table-header-owner-split-v1/CLOSEOUT_AUDIT_2026-05-25.md`, `docs/workstreams/imui-table-body-owner-split-v1/CLOSEOUT_AUDIT_2026-05-25.md` | Met for current table hotspot |
| Layering rule | `tools/gate_imui_workstream_source.py` closeout checks | Met for these slices |

## Still Not Complete

The following are not closed by this audit:

- **Real Wayland compositor acceptance.** `DW-P1-linux-003` remains `[~]` in
  `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`.
  The required proof is still a real Linux Wayland compositor run following
  `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
- **Broader DevTools GUI maturity.** The first-open route and copyable action bundle are closed, but
  richer always-available GUI execution controls and command-palette integration require later
  DevTools/diagnostics follow-ons.
- **Full perf/smoothness attribution.** Current perf evidence remains guarded through dedicated
  perf/product-chain lanes; this audit does not claim broad editor workload smoothness closure.
- **Broad porting sugar.** Plot adapter and theme picker landed as narrow owner-lane surfaces.
  Dear ImGui-style shorthand sugar remains deferred until two product routes prove the same tax.

## Verification Snapshot

Fresh checks for this refresh:

```powershell
python -m json.tool docs/workstreams/imui-editor-workbench-golden-path-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-table-body-owner-split-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
cargo fmt --check -p fret-examples -p fret-demo -p fret-ui-editor -p fret-ui-kit -p fret-imui -p fret-plot
cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast
cargo check -p fret-demo --bin imui_editor_workbench_demo
git diff --check
```

All passed. `git diff --check` reported no whitespace errors, only existing line-ending warnings for
`Cargo.lock` and `apps/fret-examples/src/lib.rs`. Cargo emitted existing warnings from
`crates/fret-ui` (`unstable-retained-bridge`) plus unrelated dead-code warnings in `fret-chart` /
`fret-plot`.

## Verdict

The IMUI-side implementation recommendations in this refresh are closed with evidence. The larger
editor-grade product-closure umbrella remains maintenance/open only because it deliberately includes
platform and product maturity requirements that cannot be honestly closed from this Windows
non-Wayland session.

The next true closure event should be either:

1. a real Linux Wayland compositor acceptance note for `DW-P1-linux-003`, or
2. a new narrow DevTools/perf/product follow-on with its own repro, gates, and evidence.
