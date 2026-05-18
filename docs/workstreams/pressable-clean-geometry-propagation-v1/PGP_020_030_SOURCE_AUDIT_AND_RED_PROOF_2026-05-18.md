# Pressable Clean Geometry Propagation v1 - PGP-020/030 Source Audit And RED Proof

Date: 2026-05-18
Status: PGP-020 complete; PGP-030 RED proof recorded

## Verdict

`Pressable` is a valid narrow candidate for the next clean-geometry execution-surface proof, but it
must be implemented in PGP-040 rather than silently assumed safe.

The current state is:

- The clean-geometry safety contract already treats `ElementInstance::Pressable(_)` as a pure
  `PreserveLocalOrigins` wrapper.
- The execution allowlist in `clean_engine_geometry_propagation_supported_element(...)` does not
  include `Pressable`.
- The focused RED test proves the resulting gap: a small width-only resize skips the root Taffy
  solve, but still reruns `Pressable` wrapper layout (`layout_nodes_performed=2`).

## Source Audit

Geometry contract:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
  - `clean_geometry_node_contract(...)` includes `ElementInstance::Pressable(_)` in the pure
    `PreserveLocalOrigins` wrapper group.
  - `clean_engine_geometry_propagation_supported_element(...)` supports `Stack`, `Semantics`,
    `Container`, `Grid`, flex variants, leaf `Spacer`, and supported text leaves, but not
    `Pressable`.

Host-widget layout and interaction flags:

- `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - `Pressable` sets `hit_testable=true` and `hit_test_children=true`.
  - `focus_traversal_children` follows `props.enabled`.
  - `is_focusable` follows `props.enabled && props.focusable`.
  - `clips_hit_test` follows `props.layout.overflow`.
  - Layout delegates to `layout_positioned_container_impl(...)` after the engine/manual-absolute
    child path.

Pointer and activation side effects:

- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`
  - Pointer down invokes component-owned hooks, prevents default pointer-down focus, captures the
    pointer, stores press tracking, sets pressed state, invalidates paint, requests redraw, and
    stops propagation.
  - Pointer up invokes hooks, releases capture, clears pressed state, computes activation from
    current bounds/down-position tolerance, optionally requests focus, and invokes activation hooks.
  - Pointer move/cancel handle stale pressed/capture cleanup.

Hover side effects:

- `crates/fret-ui/src/tree/dispatch/hover.rs`
  - `pressable_target_for_hit(...)` derives the hovered `Pressable` from the current hit chain.
  - `update_hover_state_from_hit(...)` updates hovered pressable state, marks hover-edge paint
    invalidation, and marks affected view-cache roots for rerender.

Conclusion:

- No audited side effect appears to require rerunning `Pressable` layout during a clean width-only
  bounds propagation.
- The important constraint for PGP-040 is that propagated bounds must remain authoritative before
  later hit-test, focus, hover, pressed-state, capture, and activation paths run.

## RED Proof

New focused test:

- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
  - `clean_geometry_small_resize_propagates_through_pressable_wrapper`

Command:

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast
```

Result:

- RED as expected.
- Failure:
  - `layout_engine_solves=0`, so the root Taffy solve is already skipped.
  - `layout_clean_geometry_solve_skip_rejections=0`, so the subtree is accepted by the manual
    clean-geometry proof.
  - `layout_nodes_performed=2`, which means the scheduling parent plus the `Pressable` wrapper still
    execute layout.

Failure excerpt:

```text
Pressable clean-geometry propagation should avoid re-running wrapper/subtree layout; performed=2
```

## Interaction Guard Evidence

Existing focused `Pressable` side-effect gates still pass:

```bash
cargo nextest run -p fret-ui pressable_on_activate_hook_runs_on_pointer_activation --no-fail-fast
cargo nextest run -p fret-ui pressable_on_hover_change_hook_runs_on_pointer_move --no-fail-fast
cargo nextest run -p fret-ui pressable_clears_pressed_and_releases_capture_on_move_without_buttons --no-fail-fast
```

Results:

- `pressable_on_activate_hook_runs_on_pointer_activation`: passed.
- `pressable_on_hover_change_hook_runs_on_pointer_move`: passed.
- `pressable_clears_pressed_and_releases_capture_on_move_without_buttons`: passed.

## Next Step

Proceed to PGP-040:

- Add `ElementInstance::Pressable(_)` to
  `clean_engine_geometry_propagation_supported_element(...)`.
- Re-run the RED test as the GREEN proof.
- Re-run the focused `Pressable` interaction gates and `layout_engine pressable` gate.
- Keep the code change as small as the RLO-030 `Semantics` support-matrix slice.
