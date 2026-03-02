# Evidence and Gates

## Minimum gates (local)

- `cargo fmt -p fret-ui -p fret-bootstrap -p fret-ui-gallery`
- Targeted regression tests (Phase A/B invariants):
  - `cargo nextest run -p fret-ui --lib outside_press_branch_containment_uses_child_edges_not_parent_pointers`
  - `cargo nextest run -p fret-ui --lib dismissible_outside_press_prevent_default_keeps_focus`
  - `cargo nextest run -p fret-ui --lib dismissible_outside_press_without_prevent_default_clears_focus`
- Targeted regression tests (view-cache + hover correctness):
  - `cargo nextest run -p fret-ui --lib hover_region_marks_view_cache_root_dirty_on_hover_edges`
- Full suite (when practical): `cargo nextest run -p fret-ui`
- `python3 tools/check_layering.py`
- Perf probe (optional, Phase C guardrail):
  - `cargo run -p fretboard -- diag perf ui-gallery-steady --repeat 3 --warmup-frames 5 --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-steady.v2.json --launch -- cargo run -p fret-ui-gallery --release`

## Existing regression coverage (anchors)

- Outside press routing: `crates/fret-ui/src/tree/tests/outside_press.rs`
- Escape dismissal: `crates/fret-ui/src/tree/tests/escape_dismiss.rs`
- Focus scope trap: `crates/fret-ui/src/tree/tests/focus_scope.rs`
- Declarative dismissible interactions: `crates/fret-ui/src/declarative/tests/interactions/dismissible.rs`

## Phase C anchors (snapshot PR0)

- Snapshot types + builder: `crates/fret-ui/src/tree/dispatch_snapshot.rs`
- Debug entrypoint (no behavior change): `crates/fret-ui/src/tree/ui_tree_debug/query.rs` (`debug_dispatch_snapshot`)
- Snapshot parity report: `crates/fret-ui/src/tree/ui_tree_debug/query.rs` (`debug_dispatch_snapshot_parity`)
- Outside-press routed via snapshot (PR2): `crates/fret-ui/src/tree/ui_tree_outside_press.rs` and
  `crates/fret-ui/src/tree/dispatch/window.rs`

## New artifacts (Phase A/B)

- Unit test: stale parent pointers do not break outside-press branch exclusion.
- Unit tests: outside-press default focus clearing vs `prevent_default` suppression.

## View-cache + hover regression artifacts

- Unit test: `crates/fret-ui/src/declarative/tests/layout/interactivity.rs`
  (`hover_region_marks_view_cache_root_dirty_on_hover_edges`).
- Mechanism anchors (hover edge redrawability under view-cache / retained drift):
  - Hovered HoverRegion node tracking: `crates/fret-ui/src/elements/runtime.rs` (`hovered_hover_region_node`)
  - Update hook: `crates/fret-ui/src/elements/access.rs` (`update_hovered_hover_region_with_node`)
  - Dispatch derivation: `crates/fret-ui/src/tree/dispatch/hover.rs`
- Unit test: `crates/fret-ui/src/elements/access.rs` (`hovered_hover_region_tracks_node_id_for_redraw_on_exit`)
- Scripted gate (run under view-cache env):

```powershell
cargo run -p fretboard -- diag run ui-gallery-hovercard-open `
  --env FRET_UI_GALLERY_VIEW_CACHE=1 `
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 `
  --launch -- cargo run -p fret-ui-gallery --release
```

## Wheel routing gates (pointer occlusion vs modal barrier)

- Pointer occlusion MUST allow wheel to scroll underlay content (Radix `disableOutsidePointerEvents` / GPUI
  `BlockMouseExceptScroll` ergonomics):

```powershell
cargo run -p fretboard -- diag run ui-gallery-context-menu-occlusion-wheel-pass-through `
  --check-pixels-changed ui-gallery-nav-scroll `
  --launch -- cargo run -p fret-ui-gallery --release
```

- Modal barriers MUST block wheel from reaching underlay scroll containers:

```powershell
cargo run -p fretboard -- diag run ui-gallery-modal-barrier-wheel-block `
  --check-pixels-unchanged ui-gallery-nav-scroll `
  --launch -- cargo run -p fret-ui-gallery --release
```

- Modal dialogs MUST install barrier roots and restore focus on close (and underlay press MUST be blocked):

```powershell
cargo run -p fretboard -- diag run ui-gallery-modal-barrier-focus-restore `
  --launch -- cargo run -p fret-ui-gallery --release
```

- ContextMenu keyboard invocation MUST focus the first item and restore focus to the trigger on escape:

```powershell
cargo run -p fretboard -- diag run ui-gallery-context-menu-keyboard-escape-focus-restore `
  --launch -- cargo run -p fret-ui-gallery --release
```

- Non-modal menu overlays (click-through outside press) MUST NOT steal focus back to the trigger:

```powershell
cargo run -p fretboard -- diag run ui-gallery-dropdown-nonmodal-outside-press-focus-underlay `
  --launch -- cargo run -p fret-ui-gallery --release
```
