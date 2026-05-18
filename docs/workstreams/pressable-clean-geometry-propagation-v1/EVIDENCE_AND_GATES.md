# Pressable Clean Geometry Propagation v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Starting Evidence

From `retained-layout-orchestration-v1`:

- Baseline bundle:
  `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/bundle.schema2.json`
- After bundle:
  `target/fret-diag/retained-layout-orchestration-v1-rlo030-after/1779083266980/bundle.schema2.json`
- After layout summary:
  `target/fret-diag/retained-layout-orchestration-v1-rlo030-after/layout.perf.summary.v1.json`
- RLO-030 after result:
  - `p95.total_time_us`: `3050 -> 1442`
  - `p95.layout_time_us`: `2479 -> 885`
  - `p95.layout_roots_time_us`: `2349 -> 747`
  - `p95.layout_engine_solve_time_us`: `220 -> 214`

Interpretation:

- The RLO-030 win came from avoiding retained wrapper/subtree layout around `Semantics`, not from a
  material Taffy solve-time change.
- The after layout summary still lists `Pressable` as the top layout hotspot in the local sample
  (`layout_time_us=308`), ahead of `Scroll` (`199`) and `ViewCache` (`76`).
- This evidence is local orientation. It justifies a narrow proof lane, not a broad performance
  claim.

## Source Evidence To Preserve

`Pressable` geometry and side effects are split across several owners:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
  - `clean_geometry_node_contract(...)` classifies `ElementInstance::Pressable(_)` as a pure
    `PreserveLocalOrigins` wrapper.
- `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - `Pressable` sets `hit_testable=true`, `hit_test_children=true`,
    `focus_traversal_children=p.enabled`, `is_focusable=p.enabled && p.focusable`, and
    `clips_hit_test` from `props.layout.overflow`.
  - Its layout body uses `layout_positioned_container_impl(...)` when no engine/manual absolute
    child result is available.
- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`
  - Pointer down requests capture, stores press tracking, sets pressed state, invalidates paint, and
    prevents default pointer-down focus.
  - Pointer up releases capture, clears pressed state, conditionally requests focus, and invokes
    activation hooks.
  - Pointer move/cancel paths clear stale pressed state and release capture.
- `crates/fret-ui/src/tree/dispatch/hover.rs`
  - Hover derivation walks to the current `Pressable` target and invalidates hover edge nodes.

These responsibilities are why `Pressable` cannot be treated as "just another wrapper" without a
side-effect proof.

## PGP-020/030 Audit And RED Result

Detailed note:

- `docs/workstreams/pressable-clean-geometry-propagation-v1/PGP_020_030_SOURCE_AUDIT_AND_RED_PROOF_2026-05-18.md`

Source audit result:

- `Pressable` is already in the pure wrapper clean-geometry contract.
- `Pressable` is not yet in the execution allowlist used by
  `clean_engine_geometry_propagation_supported_element(...)`.
- No audited hit-test, focus, hover, pressed-state, capture, or activation side effect appears to
  require rerunning `Pressable` layout during a clean width-only bounds propagation, but the
  propagated bounds must stay authoritative before later dispatch.

RED proof:

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast
```

Result:

- Failed as expected.
- Failure point: `layout_nodes_performed=2`.
- Interpretation: the root Taffy solve is skipped and there is no clean-geometry rejection noise,
  but the `Pressable` wrapper still falls back to `layout_node(...)` because it is missing from the
  execution allowlist.

Interaction guard checks:

```bash
cargo nextest run -p fret-ui pressable_on_activate_hook_runs_on_pointer_activation --no-fail-fast
cargo nextest run -p fret-ui pressable_on_hover_change_hook_runs_on_pointer_move --no-fail-fast
cargo nextest run -p fret-ui pressable_clears_pressed_and_releases_capture_on_move_without_buttons --no-fail-fast
```

Result: all passed.

## PGP-040 Runtime Slice Result

Runtime change:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
  - Added `ElementInstance::Pressable(_)` to the execution allowlist in
    `clean_engine_geometry_propagation_supported_element(...)`.

Green proof:

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast
```

Result:

- Passed.
- The focused `Pressable` propagation test that was RED before the runtime slice is now green.

Interaction guards:

```bash
cargo nextest run -p fret-ui pressable_on_activate_hook_runs_on_pointer_activation pressable_on_hover_change_hook_runs_on_pointer_move pressable_clears_pressed_and_releases_capture_on_move_without_buttons --no-fail-fast
cargo nextest run -p fret-ui layout_engine pressable --no-fail-fast
```

Result:

- All passed.

Interpretation:

- The minimal runtime slice was sufficient to remove the wrapper/subtree layout rerun for the
  targeted clean width-only resize.
- The change did not disturb the audited `Pressable` interaction side effects.

## PGP-050 Perf Confirmation And Closeout Result

Fresh resize-jitter capture:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json \
  --repeat 1 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_RENDERER_PERF=1 \
  --env FRET_LAYOUT_NODE_PROFILE=1 \
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 \
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Artifacts:

- Bundle:
  `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json`
- Layout summary:
  `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/layout.perf.summary.v1.json`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json \
  --sort time --top 20
```

Result:

- `p95.total_time_us=1477`
- `p95.layout_time_us=930`
- `p95.layout_engine_solve_time_us=215`
- Worst frame: `total_us=1477`, `layout_us=897`, `solve_us=211`

Layout hotspot summary:

```text
ViewCache layout_us=380 inclusive_us=723
Scroll    layout_us=205 inclusive_us=331
Flex      layout_us=83  inclusive_us=122
```

Historical RLO-030 after summary for comparison:

```text
Pressable layout_us=308 inclusive_us=684
Scroll    layout_us=199 inclusive_us=287
ViewCache layout_us=76  inclusive_us=375
```

Diff against the RLO-030 after bundle:

```bash
target/release/fretboard-dev diag stats --diff \
  target/fret-diag/retained-layout-orchestration-v1-rlo030-after/1779083266980/bundle.schema2.json \
  target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json \
  --sort time --top 10
```

Highlights:

- `p95.total_time_us`: `1442 -> 1477` (`+2.4%`)
- `p95.layout_time_us`: `885 -> 930` (`+5.1%`)
- `p95.layout_engine_solve_time_us`: `214 -> 215` (`+0.5%`)
- `p95.prepaint_time_us`: `262 -> 239` (`-8.8%`)
- `p95.paint_time_us`: `367 -> 351` (`-4.4%`)

Interpretation:

- The targeted `Pressable` issue is closed: `Pressable` moved off the worst-frame layout hotspot
  list after the PGP-040 allowlist change.
- The overall local tail is roughly flat in this single-run comparison. This lane should not claim a
  universal frame-time win.
- Fresh attribution now points at `ViewCache` first, then `Scroll`; both need separate proof lanes
  because their cache, viewport, clipping, and input semantics differ from `Pressable`.

## Smallest Current Repro

Historical resize-jitter repro from RLO:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json \
  --repeat 1 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_RENDERER_PERF=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/pressable-clean-geometry-propagation-v1-baseline \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Inspect the bundle with:

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20
```

Smallest unit-level target to add in PGP-030:

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast
```

## Gate Set

### Planning Gates

```bash
python3 -m json.tool docs/workstreams/pressable-clean-geometry-propagation-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
git diff --check
```

What they prove:

- `WORKSTREAM.json` is parseable.
- The catalog count and directory index cover the new dedicated lane.
- Markdown edits do not introduce whitespace errors.

### Targeted Layout Gate

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast
```

What it should prove after PGP-030/PGP-040:

- Width-only clean geometry can propagate through a `Pressable` wrapper without a Taffy root solve.
- Descendant bounds remain authoritative after propagation.
- The proof does not rely on `Scroll` or `ViewCache` behavior.

### Pressable Interaction Gate

```bash
cargo nextest run -p fret-ui pressable_on_activate_hook_runs_on_pointer_activation pressable_on_hover_change_hook_runs_on_pointer_move pressable_clears_pressed_and_releases_capture_on_move_without_buttons --no-fail-fast
```

What it proves:

- Activation, hover, pressed-state cleanup, and pointer capture behavior still route through current
  bounds after any propagation change.

### Broader Mechanism Gate

```bash
cargo nextest run -p fret-ui layout_engine pressable --no-fail-fast
python3 tools/check_layering.py
cargo fmt --check
git diff --check
```

What it proves:

- Layout engine and `Pressable` mechanism tests agree with the change.
- The implementation stays in the `fret-ui` mechanism layer.
- Formatting and diff hygiene are clean.

## Evidence Anchors

- `docs/workstreams/pressable-clean-geometry-propagation-v1/DESIGN.md`
- `docs/workstreams/pressable-clean-geometry-propagation-v1/TODO.md`
- `docs/workstreams/pressable-clean-geometry-propagation-v1/MILESTONES.md`
- `docs/workstreams/pressable-clean-geometry-propagation-v1/HANDOFF.md`
- `docs/workstreams/pressable-clean-geometry-propagation-v1/CLOSEOUT_AUDIT_2026-05-18.md`
- `docs/workstreams/pressable-clean-geometry-propagation-v1/PGP_020_030_SOURCE_AUDIT_AND_RED_PROOF_2026-05-18.md`
- `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-layout-orchestration-v1/EVIDENCE_AND_GATES.md`
- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/declarative/host_widget/layout.rs`
- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`
- `crates/fret-ui/src/tree/dispatch/hover.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
- `crates/fret-ui/src/declarative/tests/interactions/pressable.rs`

## Notes

Do not add `Pressable` to the execution allowlist based only on the pure wrapper classification.
The proof must show that skipping subtree layout is not skipping the authoritative update point for
hit testing, focus traversal, hover edges, pressed state, capture release, or activation geometry.
