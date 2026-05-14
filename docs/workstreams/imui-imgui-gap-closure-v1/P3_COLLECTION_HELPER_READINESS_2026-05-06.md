# P3 Collection Helper Readiness - 2026-05-06

Status: readiness audit; no collection helper follow-on opened yet
Last updated: 2026-05-13

## Decision

Do not extract a general IMUI collection helper from
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` yet.

The proof is valuable because it demonstrates an editor-grade collection surface, but most of its
logic is intentionally application-owned:

- visible asset ordering,
- selected/active tile readouts,
- keyboard active-tile ownership,
- select-all / duplicate / delete / rename command packaging,
- inline rename focus restoration,
- box-select geometry and append/replace policy,
- context-menu target selection,
- scroll-anchor-preserving zoom.

Current public/shared pieces are already at the right layer:

- `ecosystem/fret-ui-kit/src/imui/multi_select.rs` owns the minimal `ImUiMultiSelectState` plus
  click/range/toggle `multi_selectable` behavior.
- `ecosystem/fret-ui-kit/src/recipes/imui_sortable.rs` owns reorder packaging over typed IMUI
  drag/drop without taking app mutation ownership.
- `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs` owns drag-preview presentation policy
  without owning app preview content.

## Dear ImGui Reference Axes

The relevant Dear ImGui collection surface is broader than one helper:

- `BeginMultiSelect()` / `EndMultiSelect()` produce request streams rather than directly owning app
  collections.
- `ImGuiSelectionBasicStorage` and `ImGuiSelectionExternalStorage` are optional convenience storage
  helpers, not required app state owners.
- `ImGuiMultiSelectFlags_BoxSelect1d` and `BoxSelect2d` change selection interaction and clipping
  tradeoffs.
- deletion flows use pre-loop / post-loop selection repair so focus can move to a stable survivor.
- demos combine multi-select, clipper, scrolling child regions, drag/drop, context menus, and
  asset-browser-specific rename/delete behavior.

## Current Fret Read

- Confident: the current IMUI collection proof is one app-owned proof surface, not a shared helper
  proof. Evidence: `apps/fret-examples/tests/imui_editor_collection_box_select_surface.rs`
  explicitly rejects `fret_ui_kit::imui::collection_box_select` and keeps box-select local.
- Confident: Fret already extracted the low-level reusable pieces that are proven by multiple
  call sites: multi-select state, sortable row signals, and drag-preview ghosts.
- Likely: `fret-node` is not a valid second proof for an IMUI collection helper. It has marquee and
  multi-selection behavior, but the semantics depend on graph-space coordinates, nodes, edges,
  groups, store-backed views, and node-editor interaction policy.
- Likely: a future helper should look more like a request/transition vocabulary than a monolithic
  "asset collection widget". Dear ImGui's own design separates request flow from app storage.
- Unclear: virtualized collection + box-select requires a dedicated proof before public API design.
  The current collection proof is scrollable and grid-like, but it is not yet a clipped virtual-grid
  conformance surface.

## Follow-On Threshold

Open a narrow collection follow-on only when one of these targets has two first-party proof
surfaces, a repro, and a gate:

1. **Multi-select request vocabulary**: if another IMUI table/tree/list/collection needs the same
   keyboard, range, delete/refocus, and select-all transitions as the asset collection.
2. **Box-select grid recipe**: if another IMUI collection uses the same scroll-local rectangle
   hit-testing, append/replace policy, pointer capture, and marquee rendering.
3. **Virtualized selection repair**: if a virtual list/grid proof needs Dear ImGui-style
   deletion/refocus and range-source preservation across clipped rows.
4. **Collection command package**: only if two asset-like collections share duplicate/delete/rename
   semantics. Otherwise these remain application commands.

Do not use `fret-node` alone as the second proof for any of these. It can inform geometry and
selection-policy questions, but its domain model is too different to freeze an IMUI API.

## Recommended Next Slice

Keep `collection helper readiness` candidate-only for now.

If a follow-on becomes justified, start with `imui-multi-select-request-v1` or
`imui-box-select-grid-recipe-v1`, not a broad `collection_view` helper.

Suggested readiness gates:

```powershell
python tools/gate_imui_editor_collection_source.py
cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --test imui_sortable_recipe_smoke --test imui_drag_preview_smoke --no-fail-fast
```

## Gate Results

2026-05-06 local results:

- `cargo nextest run -p fret-examples --test imui_editor_collection_modularization_surface --test imui_editor_collection_command_package_surface --test imui_editor_collection_context_menu_surface --test imui_editor_collection_keyboard_owner_surface --test imui_editor_collection_select_all_surface --test imui_editor_collection_rename_surface --test imui_editor_collection_delete_action_surface --test imui_editor_collection_box_select_surface --test imui_editor_collection_zoom_surface --no-fail-fast`
  passed: 9 app-owned collection proof tests passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --test imui_sortable_recipe_smoke --test imui_drag_preview_smoke --no-fail-fast`
  passed: 4 public helper smoke tests passed.

2026-05-13 gate refresh:

- `python tools/gate_imui_editor_collection_source.py` passed and is now the active source gate for
  the same marker checks.
