# Text Clean Geometry Stability v1 Evidence And Gates

Date: 2026-05-21

Closeout status: passed as a boundary and diagnostics record. No wrapped-text eligibility change
ships from this lane.

## Initial Boundary Evidence

Focused gate:

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow clean_geometry_small_resize_skips_nowrap_text_width_delta_when_height_stable clean_geometry_small_resize_rejects_nowrap_text_height_delta --no-fail-fast
```

Expected outcomes:

- `clean_geometry_small_resize_rejects_auto_height_text_reflow` keeps width-dependent text on the
  authoritative path and reports `text_reflow / Text`.
- `clean_geometry_small_resize_skips_nowrap_text_width_delta_when_height_stable` accepts cached
  `TextWrap::None` text with stable height and unchanged measured size.
- `clean_geometry_small_resize_rejects_nowrap_text_height_delta` rejects nowrap text when its height
  changes.

## Source Evidence

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`:
  - `Text`, `StyledText`, and `SelectableText` use `NoWrapTextCachedMetrics`.
  - `clean_nowrap_text_cached_metrics_supported` rejects any text outside `TextWrap::None`,
    `TextOverflow::Clip`, and `TextAlign::Start`.
  - Changed height, missing cache, mismatched cached size, or changed fingerprint all reject as
    `text_reflow`.
- `ecosystem/fret-ui-shadcn/src/card.rs`:
  - `CardDescription::new(text)` authors text as `.w_full().wrap(TextWrap::Word)`.
- `apps/fret-ui-gallery/src/ui/content.rs`:
  - the preview card description uses `CardDescription::new("Interactive preview for validating behaviors.")`.
- `docs/workstreams/scroll-optimization-v1/HANDOFF.md`:
  - names text computed-box / line-break stability as a separate follow-on.

## Eligibility Matrix Evidence

`DESIGN.md` now records the current text clean-geometry eligibility matrix:

- accepted: `Text`, `StyledText`, or `SelectableText` with `TextWrap::None`,
  `TextOverflow::Clip`, `TextAlign::Start`, unchanged height, matching cached measured size, and an
  unchanged wrap-none measure fingerprint,
- rejected as `text_reflow`: wrapped text, non-clip overflow, non-start alignment, height-changing
  text, missing cache, cached-size mismatch, and stale content/style/font-stack/scale fingerprints,
- workstream-local for now; promote to a runtime contract doc only when behavior changes.

## Diagnostics Shape Evidence

The T2 diagnostics audit found that the existing debug and bundle projection already exposed:

- rejection `reason`,
- rejected node,
- rejected element id,
- rejected element kind,
- rejected element path when diagnostics are enabled.

That was enough for generic attribution but too coarse for future text proof work because all text
sub-causes surfaced as `text_reflow`. The runtime debug record and diagnostics schema now add an
optional, additive `detail` field. The first focused text proof records
`detail=text_wrap_not_none` for wrapped `TextWrap::Word` / default wrapped text rejection.

Source anchors:

- `crates/fret-ui/src/tree/debug/layout.rs`
- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/layout_paint_hotspot_diagnostics.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
- `clean_geometry_rejection_detail_is_additive` proves the diagnostics schema serializes `detail`
  and still accepts legacy JSON without it.

Artifact proof:

```bash
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json --dir target/fret-diag/text-clean-geometry-detail-20260521-r1 --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --features gallery-dev
```

Result: passed. Bundle:
`target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/1779306373288/bundle.schema2.json`.
Share artifacts:
`target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/share/1779306373288.zip`
and
`target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/1779306373288/ai.packet`.

Structured bundle query confirmed `clean_geometry_solve_skip_rejection` includes the additive
`detail` field in real UI Gallery resize diagnostics:

- frame 208/211/214, root `Semantics`: `reason=text_reflow`, `detail=text_wrap_not_none`,
  `element_kind=Text`,
- frame 208/211/214, root `Stack`: `reason=text_reflow`, `detail=text_overflow_not_clip`,
  `element_kind=Text`.

This is diagnostics-shape evidence only. It does not make wrapped text eligible for clean-geometry
resize propagation and does not close the future T3/T4 performance proof.

## Required Gate Set

- `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow clean_geometry_small_resize_skips_nowrap_text_width_delta_when_height_stable clean_geometry_small_resize_rejects_nowrap_text_height_delta --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics clean_geometry_rejection_detail_is_additive --no-fail-fast`
- `cargo fmt -p fret-ui -p fret-bootstrap --check`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json --dir target/fret-diag/text-clean-geometry-detail-20260521-r1 --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
- `python -m json.tool docs/workstreams/text-clean-geometry-stability-v1/WORKSTREAM.json`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## Closeout Evidence

The closeout audit is recorded in
`docs/workstreams/text-clean-geometry-stability-v1/CLOSEOUT_AUDIT_2026-05-21.md`.
