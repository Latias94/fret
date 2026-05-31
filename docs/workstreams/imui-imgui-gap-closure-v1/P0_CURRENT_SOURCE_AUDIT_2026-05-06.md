# P0 Current Source Audit - 2026-05-06

Status: Current baseline for `imui-imgui-gap-closure-v1`

## Evidence Read

This audit was refreshed from current repo sources and the local Dear ImGui mirror:

- Fret source anchors:
  - `ecosystem/fret-imui/src/lib.rs`
  - `ecosystem/fret-imui/src/frontend.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-editor/src/imui.rs`
  - `apps/fret-cookbook/examples/imui_action_basics.rs`
  - `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
  - `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
  - `apps/fret-examples/src/imui_editor_workbench_demo.rs`
  - `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs`
  - `docs/workstreams/imui-editor-workbench-golden-path-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- Dear ImGui anchors:
  - `repo-ref/imgui/imgui.h`
  - `repo-ref/imgui/imgui.cpp`
  - `repo-ref/imgui/imgui_draw.cpp`
  - `repo-ref/imgui/imgui_demo.cpp`
  - `repo-ref/imgui/docs/BACKENDS.md`

## Current Fret IMUI Capabilities

The stack is materially beyond the old prototype stage:

- `fret-imui` is a thin immediate authoring facade and keeps platform/renderer dependencies out.
- `fret-ui-kit::imui` owns the broader helper surface:
  - identity: `push_id`, keyed/unkeyed iteration,
  - layout: horizontal, vertical, grid, scroll, table, virtual list, child region,
  - interaction controls: button, small button, selectable, boolean controls, combo, tree node,
    collapsing header, slider, text input, textarea, picker recipes,
  - popup/floating: menu bar, menu/submenu, popup menu/modal, tooltip, floating area/window,
  - editor interaction helpers: drag source/drop target, multi-select state,
  - draw-list style debug drawing with later 2026-05 follow-ons for clipping, paths, channel split,
    triangle mesh, command metadata, and image variants.
- `fret-ui-editor::imui` now adapts editor controls and composites instead of reimplementing them:
  - text field, checkbox, color edit, drag value, axis drag value, numeric input, slider,
    enum select, mini search box, text assist field, icon button, status badge,
    vector/transform editors, property group/grid, virtualized property grid, inspector panel.
- Cookbook proof surfaces cover action interop, debug drawing, and editor controls through
  `cookbook-imui`.
- The canonical product-facing editor workbench route is now
  `cargo run -p fret-demo --bin imui_editor_workbench_demo`; it mounts the editor-notes workflow
  directly and keeps `imui_editor_proof_demo`, `workspace_shell_demo`, and docking demos as
  supporting proof surfaces.

## Stale or Partially Superseded Older Conclusions

The standalone v2 parity audit is still useful for architecture and phase ordering, but it now
understates several areas:

- Debug draw breadth is newer than that audit's practical read. Multiple 2026-05 closeouts added
  path builders, channel split, mesh/quad/image variants, command metadata, and diag smoke
  coverage.
- Color edit depth is newer than that audit's practical read. Multiple 2026-05 closeouts added
  Dear ImGui-style alpha policy, popup splitting, picker variants, drag/drop payloads, history,
  palette customization, copy-as context menus, tooltip previews, and reference previews.
- Text input depth is newer than that audit's practical read. Current closeouts cover read-only,
  select-all-on-focus, multiline Tab policy, explicit identity, filters, picker recipes, keyboard
  navigation, accessibility, and test splitting.
- The broad "helper breadth" problem is now narrower. The remaining question is not whether Fret
  can do immediate-mode controls; it is whether the user-facing golden path feels coherent and
  whether any compatibility sugar pays for itself across multiple proof surfaces.

## Current Gap Matrix

| Priority | Gap | Current read | Owner | First gate |
| --- | --- | --- | --- | --- |
| P0 | Source-of-truth drift | Old parity notes and current source have diverged. | docs/workstreams | `python tools/check_workstream_catalog.py` |
| P1 | Fearless cleanup/deletion | The stack has many narrow closeouts; stale teaching paths can keep old surfaces alive by accident. | docs + examples + ecosystem owners | `python tools/gate_imui_facade_teaching_source.py` |
| P2 | User-usable golden path | Closed on the IMUI side: `imui_editor_workbench_demo` is the canonical product-facing editor route, with cookbook/docs/discovery pointing to it and older proof demos demoted to supporting surfaces. | `fret`, cookbook, `fret-ui-editor`, examples | `imui_editor_workbench_golden_path_surface` + `cargo check -p fret-demo --bin imui_editor_workbench_demo` |
| P2 | Workbench product closure | The workbench route, style/theme picker integration, Demo/Metrics/Debug route, ListBox, plot adapter, and table owner slices are closed with evidence; full Dear ImGui-class product closure still depends on external owner lanes for real-host Wayland, broader DevTools GUI maturity, perf/smoothness attribution, and broad porting sugar. | workspace/docking/editor/devtools/perf | product-chain gates + owner-lane closeout audits |
| P3 | Multi-window/backend hand-feel | Still a top parity risk: hovered viewport, peek-behind, transparent payload, mixed-DPI, release/cancel paths. | runner/backend + `fret-docking` | `imui-p3-multiwindow-parity` campaign |
| P3 | Diagnostics ambient usability | Scripted diagnostics are strong, but the always-open Demo/Metrics/Debug culture is less immediate than Dear ImGui. | `fret-diag`, `fret-devtools`, bootstrap | devtools first-open smoke |
| P3 | Porting ergonomics | Current proof surfaces already cover most authoring friction with explicit `PropertyGrid`, `row_with`, `horizontal_with_options`, `child_region_with_options`, closure-scoped `same_line` helpers, and stable `id_source` / `test_id` wiring. `SameLine` is now a narrow proven teaching-surface helper; item-width stacks, next-item width defaults, and label-ID sugar remain candidate-only. | `fret-imui` / `fret-ui-kit::imui` | same-line cookbook proof + two-proof helper-readiness rule |
| P3 | Child-region depth | Fret has child region chrome, but not full `BeginChild()` semantics such as axis-specific resize, clipping return, and nav flattening. | `fret-ui-kit::imui` if proven | child-region focused tests |
| P3 | Collection helper readiness | The editor collection proof is strong but app-owned. Current shared pieces are already narrow (`ImUiMultiSelectState`, sortable row recipe, drag-preview recipe). `fret-node` has marquee/multi-selection behavior, but its graph-space node/edge/group semantics are not a second IMUI collection proof. | app proof first, shared helper later | collection proof-surface gates |
| P4 | Runtime public surface | No current evidence says `crates/fret-ui` must widen for Dear ImGui parity. | runtime ADR only if proven | ADR + focused tests |

## Fearless Refactor / Delete Candidates

These are candidates, not approved edits. Each requires a narrow follow-on with a gate:

- Delete or demote any first-party docs/examples that still teach raw `fret_imui::imui_raw(...)`
  as the default app-facing path instead of `fret::imui`.
- Delete duplicate public aliases after a canonical helper name exists and source-policy tests prove
  no teaching surface uses the alias.
- Keep `fret-ui-editor::imui` as a thin adapter; move or delete any code that starts reimplementing
  editor control behavior there.
- Split or further modularize very large `fret-ui-kit::imui` implementation files only when the
  public surface can remain frozen and focused tests cover the moved behavior.
- Delete stale parity claims from active docs by replacing them with status notes, not by rewriting
  historical closeout evidence.

## Recommended Next Slices

1. Keep the canonical workbench route current: when editor workflow depth changes, update
   `imui_editor_workbench_demo` and its golden-path surface test first, leaving older proof demos as
   supporting surfaces.
2. P1 cleanup scan: compare `fret::imui` public teaching imports against examples/docs and delete
   obsolete direct-crate teaching paths where gates allow it.
3. P3 porting sugar proposal: keep the closure-scoped `same_line` helper narrow; only after two
   product surfaces show repeated friction, propose item-width, next-item width, or label-ID sugar
   in `fret-ui-kit::imui`.
4. P3 diagnostics proposal: use the existing
   `docs/workstreams/standalone/diag-devtools-gui-refresh-v1.md` follow-on to keep the
   `ShowDemoWindow + ShowMetricsWindow` equivalent on the diagnostics-consumer lane, not as
   runtime clutter.
5. P3 multi-window follow-on: continue in `docking-multiwindow-imgui-parity`, not in generic IMUI.

## Decision

The next work should not be a large runtime rewrite. The immediate-mode substrate is adequate.
The lane should now drive narrow, evidence-backed cleanups and product-proof closure.
