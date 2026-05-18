# CPT-030/CPT-040 Owner Proof - 2026-05-18

Status: Complete
Last updated: 2026-05-18

## Verdict

The 360ms `Canvas` paint tail was real row paint work, but the owner was upstream layout sizing in
`fret-ui`, not renderer noise, `ViewCache`, or code-editor row painting policy.

The failing path was:

```text
outer scroll overflow probe
  -> positioned/pass-through wrapper
  -> TextInputRegion
  -> windowed Scroll
  -> PointerRegion
  -> tall Canvas content
```

`layout_positioned_container_impl` measured non-absolute `Fill` / `Fraction` children during a
probe, then reused that measured size for final static/relative child layout. Under an outer scroll
overflow probe, the measured height could become the tall child content height. The inner windowed
scroll then received a viewport equal to its full content, so it correctly painted every visible row
according to the wrong viewport.

The runtime fix keeps probe measurement for container intrinsic size, but final static/relative
child layout now resolves `Fill` and `Fraction` child axes against the wrapper base size.
`Auto` and `Px` child axes still use their measured size.

Implementation anchor:

- `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`
  - `static_child_size_for_base(...)`
  - `layout_positioned_container_impl(...)`

## CPT-030 Repro Evidence

Bundle:

- `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030/1779092655064/bundle.schema2.json`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030/1779092655064/bundle.schema2.json \
  --sort time --top 20
```

Observed summary:

- `time p50/p95 (us): total=722/380102, layout=35/1140, prepaint=16/2575, paint=669/376387`
- Worst frame: `total=380102us`, `layout=1140us`, `prepaint=2575us`, `paint=376387us`
- `paint_widget.hotspots canvas.top_exclusive_us(p50/p95/max)=375840/375989/375989`
- `code_editor.paint_perf` worst visible range: `visible(start/end/rows)=0/20003/20004`
- `code_editor.paint_perf.surface` worst row paint: `row_paint=359962us`
- Worst frame row text paint: `text=321599us`

The row-paint counters proved that the `Canvas` tail was real code-editor row work, but not that
code-editor policy was the root owner. The structural scroll evidence showed the upstream error:

```bash
jq '.. | objects | select(.test_id? == "ui-gallery-code-editor-torture-viewport") |
  {test_id, viewport_h, content_h}' \
  target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030/1779092655064/bundle.schema2.json
```

Result:

```json
{"test_id":"ui-gallery-code-editor-torture-viewport","viewport_h":320064.0,"content_h":320064.0}
```

The inner windowed scroll viewport was the full document height, so the windowed surface painted
`20004` rows.

## Focused Runtime Proof

Regression tests:

```bash
cargo nextest run -p fret-ui scroll_viewport_for_tall_canvas_child
```

Fresh local result:

```text
2 tests run: 2 passed, 1019 skipped
```

Test anchors:

- `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
  - `text_input_region_preserves_fill_scroll_viewport_for_tall_canvas_child`
  - `nested_page_scroll_preserves_inner_windowed_scroll_viewport_for_tall_canvas_child`

The first test proves the direct `TextInputRegion -> Scroll -> Canvas` path keeps a `Fill` viewport
bounded to its parent. The second test mirrors the UI Gallery nested-page structure, including an
outer page scroll and the `PointerRegion` wrapper used by the code-editor surface, and locks the
inner windowed scroll viewport to the panel height rather than the tall canvas content height.

## CPT-040 After Evidence

Bundle:

- `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt040/1779099328829/bundle.schema2.json`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt040/1779099328829/bundle.schema2.json \
  --sort time --top 20
```

Observed summary:

- `time p50/p95 (us): total=698/1425, layout=39/761, prepaint=124/286, paint=393/418`
- Worst frame: `total=1425us`, `layout=753us`, `prepaint=274us`, `paint=398us`
- `paint_widget.hotspots canvas.top_exclusive_us(p50/p95/max)=125/134/134`
- `code_editor.paint_perf` max rows: `painted=289`, `replayed=289`
- `code_editor.paint_perf.surface_p95_us(callback/row_paint/non_row/row_callback_gap/hook)=132/113/20/16/1`

Structural scroll evidence after the fix:

```json
{"test_id":"ui-gallery-code-editor-torture-viewport","viewport_h":518.0,"content_h":320064.0}
```

The inner windowed scroll now sees a bounded viewport and paints only the visible row window. The
360ms `Canvas` tail is eliminated in the same local repro script.

## Decision

Close this lane with the `fret-ui` positioned-container layout fix. Do not split renderer or
code-editor row-surface optimization work from this evidence set. Future optimization work should
start from a new bundle only if a bounded viewport still shows a real paint or layout owner.
