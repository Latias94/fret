# Resize Layout Proof — 2026-04-02

Status: Closed

This note records the launched resize/layout proof artifacts that close `M5` for
`default-app-productization-fearless-refactor-v1`.

## Scope

Goal:

- prove that `todo_demo` survives compact-to-regular resize roundtrips without leaving key chrome
  outside the window,
- promote that proof into a reusable diagnostics artifact rather than relying on manual window
  dragging,
- and leave explicit evidence for the remaining app-authoring surface gap discovered during the
  proof run.

## Commands used

Immediate layout roundtrip:

- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/tooling/todo/todo-resize-roundtrip-immediate-layout.json --dir target/diag/todo-resize-roundtrip-immediate-layout-m5d --include-screenshots --exit-after-run --launch -- cargo run -p fret-demo --bin todo_demo`

Footer in-window roundtrip:

- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/tooling/todo/todo-resize-roundtrip-footer-within-window.json --dir target/diag/todo-resize-roundtrip-footer-within-window-m5 --include-screenshots --exit-after-run --launch -- cargo run -p fret-demo --bin todo_demo`

Focused regression checks:

- `cargo nextest run -p fret-examples todo_demo_prefers_default_app_surface simple_todo_demo_prefers_default_app_surface selected_view_runtime_examples_prefer_grouped_state_actions_and_effects todo_demo_registers_vendor_icons_used_by_layout todo_demo_responsive_layout_prefers_compact_footer_and_inline_actions_on_narrow_width todo_demo_responsive_layout_gives_roomy_shells_more_vertical_headroom todo_demo_responsive_layout_centers_card_once_viewport_is_large_enough todo_demo_responsive_layout_keeps_inline_row_actions_for_non_hover_pointers`
- `rustfmt --check apps/fret-examples/src/lib.rs apps/fret-examples/src/todo_demo.rs ecosystem/fret/src/view.rs --edition 2024`
- `git diff --check -- apps/fret-examples/src/lib.rs apps/fret-examples/src/todo_demo.rs ecosystem/fret/src/view.rs tools/diag-scripts/tooling/todo/todo-resize-roundtrip-immediate-layout.json tools/diag-scripts/tooling/todo/todo-resize-roundtrip-footer-within-window.json docs/README.md docs/workstreams/default-app-productization-fearless-refactor-v1`

## Result summary

### 1. Compact-to-regular resize roundtrip now has a promoted proof artifact

Script:

- `tools/diag-scripts/tooling/todo/todo-resize-roundtrip-immediate-layout.json`

Observed result:

- `PASS (run_id=1775110315967)`

Artifact roots:

- packed run: `target/diag/todo-resize-roundtrip-immediate-layout-m5d/share/1775110315967.zip`
- layout sidecar:
  `target/diag/todo-resize-roundtrip-immediate-layout-m5d/1775110316121-todo-resize-roundtrip-immediate-layout.layout/layout.taffy.v1.json`
- screenshot:
  `target/diag/todo-resize-roundtrip-immediate-layout-m5d/screenshots/1775110316141-todo-resize-roundtrip-immediate-layout/window-4294967297-tick-14-frame-16.png`

What is asserted:

- `todo_demo.root` stays within the window after shrinking to `420x560` and restoring to
  `680x760`,
- `todo_demo.clear_done` stays within the window,
- `todo_demo.filter.completed` stays within the window.

### 2. Footer-specific resize regression also has a promoted proof artifact

Script:

- `tools/diag-scripts/tooling/todo/todo-resize-roundtrip-footer-within-window.json`

Observed result:

- `PASS (run_id=1775110434089)`

Artifact roots:

- packed run: `target/diag/todo-resize-roundtrip-footer-within-window-m5/share/1775110434089.zip`
- layout sidecar:
  `target/diag/todo-resize-roundtrip-footer-within-window-m5/1775110434693-todo-resize-roundtrip-footer-within-window.layout/layout.taffy.v1.json`
- screenshot:
  `target/diag/todo-resize-roundtrip-footer-within-window-m5/screenshots/1775110434715-todo-resize-roundtrip-footer-within-window/window-4294967297-tick-67-frame-69.png`

What is asserted:

- the same root/footer selectors remain in bounds after a slower shrink/restore roundtrip with
  additional settle frames,
- future regressions now have both screenshot and layout-tree evidence rather than only a visual
  complaint.

### 3. The proof run also surfaced one real authoring-diagnostics gap

During both launched proofs, diagnostics still emitted:

- `use_state called multiple times per frame at the same callsite`

After landing `#[track_caller]` on `ecosystem/fret/src/view.rs::local_with(...)`, the warnings now
resolve to the actual grouped-local construction lines in `apps/fret-examples/src/todo_demo.rs`
instead of pointing at framework internals.

Interpretation:

- this is no longer an opaque framework-internal warning,
- but the default grouped-local story (`TodoLocals::new(cx)`) still trips the current per-frame
  repeated-call heuristic during launched diagnostic runs,
- so the remaining issue is an authoring-diagnostics false-positive / over-broad warning
  heuristic, not a resize correctness failure.

### 4. Row-local list authoring was tightened while auditing the proof surface

`todo_demo` now renders rows through `ui::for_each_keyed_with_cx(...)` so row-local builder work
lands in the keyed child scope that the `ui` helpers already recommend for dynamic lists with
row-scoped subtree assembly.

This keeps the example aligned with the framework's own keyed-list guidance even though it did not
fully close the diagnostics warning above.

## Evidence anchors

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `tools/diag-scripts/tooling/todo/todo-resize-roundtrip-immediate-layout.json`
- `tools/diag-scripts/tooling/todo/todo-resize-roundtrip-footer-within-window.json`

## ADR posture

No new ADR is required for this closeout note.

Reason:

- the shipped change promotes diagnostics evidence and tightens first-party authoring/gating,
- but it does not reopen the default local-state contract, resize contract, or page-shell ownership
  decision.

