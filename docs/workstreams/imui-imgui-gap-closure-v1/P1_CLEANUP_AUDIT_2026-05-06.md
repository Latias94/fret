# P1 Cleanup Audit - 2026-05-06

Status: Current P1 cleanup audit for `imui-imgui-gap-closure-v1`

## Teaching Import Cleanup

The first P1 cleanup pass keeps first-party IMUI teaching and proof surfaces on the app-facing
`fret::imui` facade:

- `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs` now routes table sort state through
  `kit::TableSortDirection`.
- `apps/fret-examples/src/workspace_shell_demo.rs` now routes pane-proof IMUI option types through
  `kit::{ChildRegionOptions, ScrollOptions, HorizontalOptions}`.
- `apps/fret-examples/src/imui_editor_proof_demo.rs` and
  `apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now route golden-proof IMUI
  option/state types through `fret::imui`.

The source gates now forbid direct `fret_ui_kit::imui` imports from returning to the default
teaching/product proof surfaces above. Recipe-layer imports such as
`fret_ui_kit::recipes::imui_drag_preview` remain explicit because they are not the root IMUI
facade.

2026-06-03 drift guard refresh: `tools/gate_imui_facade_teaching_source.py` now defines an explicit
active teaching path set for cookbook IMUI lessons, workbench/proof surfaces, examples docs,
cookbook docs, `ecosystem/fret` README, and the root README. Those paths reject direct
`fret_imui::` and `fret_ui_kit::imui::` teaching imports, while the editor cookbook proof also
rejects direct `fret_ui_editor` imports so it remains on `fret::imui::editor`. The same refresh
aligns the proof-demo helper checks with the current `proof_helpers.rs` owner instead of stale
root-file definitions.

## Allowed Direct References

These references are intentionally not cleanup targets in this pass:

- `apps/fret-examples/src/imui_node_graph_demo.rs` remains a compatibility-only retained-bridge
  proof and is explicitly documented as non-default downstream guidance.
- `ecosystem/fret-imui/src/tests/*` and `ecosystem/fret-ui-kit/tests/*` may import direct crates
  because they test the owning crates.
- Historical workstream docs may keep older `fret_imui::` and `fret_ui_kit::imui::` references as
  evidence, as long as active docs point readers at the current `fret::imui` lane.

## Editor Adapter Check

`ecosystem/fret-ui-editor/src/imui.rs` remains a thin adapter over declarative editor controls and
composites. It uses one local `add_editor_element(...)` helper and forwards controls through
`into_element(...)`; it does not reimplement editor control behavior. No `fret-ui-editor::imui`
code refactor is needed for this P1 pass.

## Duplicate Helper Alias Check

No additional public helper alias should be deleted in this lane:

- Historical duplicate names such as `select_model_ex`, `window_ex`, `window_open_ex`,
  `floating_area_show_ex`, `begin_disabled`, `button_adapter`, and `checkbox_model_adapter` are
  already gone from active source. Current references are historical docs, source gates, or tests
  proving those names stay absent.
- `ecosystem/fret-ui-kit/src/imui/adapters.rs` is not a duplicate helper family. It is a
  contract-only external-adapter seam (`AdapterSignal*`, `AdapterSeamOptions`, and
  `report_adapter_signal(...)`) with smoke tests.
- `*_with_options(...)` helpers are the canonical paired API shape for default options versus
  explicit options. They are not compatibility aliases and should not be collapsed without stronger
  first-party proof.

Decision: close the P1 alias-deletion check with a no-delete verdict.

## Large Owner Candidates

The large implementation files are in `ecosystem/fret-ui-kit/src/imui/`, not
`fret-ui-editor::imui`:

| File | Approx size | Current read |
| --- | ---: | --- |
| `debug_draw_controls.rs` | 139 KB | Highest split candidate; needs a dedicated no-public-API follow-on. |
| `facade_writer.rs` | 66 KB | Large but central public writer glue; split only with focused smoke gates. |
| `floating_window_on_area.rs` | 40 KB | Plausible owner split after multi-window/drag gates are named. |
| `disclosure_controls.rs` / `menu_family_controls.rs` / `table_controls.rs` / `text_controls.rs` / `popup_overlay.rs` | 22-29 KB | Reviewable enough for now; split only when editing their behavior. |

Recommended next owner-split lane: `imui-debug-draw-owner-split-v1`, with a frozen public API,
`cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`
when available, plus the current source gates.
