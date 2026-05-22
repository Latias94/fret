# Text Clean Geometry Stability v1 TODO

Date: 2026-05-21

## T0 - Boundary Lane Creation

- [x] Split the text proof out of `scroll-optimization-v1`.
- [x] Record the current accepted and rejected text clean-geometry cases.
- [x] Bind the lane to the focused text clean-geometry gate:
  `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow clean_geometry_small_resize_skips_nowrap_text_width_delta_when_height_stable clean_geometry_small_resize_rejects_nowrap_text_height_delta --no-fail-fast`.
- [x] Cross-link the parent scroll lane to this follow-on.

## T1 - Eligibility Matrix

- [x] Audit `clean_geometry_node_contract` and `clean_nowrap_text_cached_metrics_supported` into a
  compact eligibility matrix covering `Text`, `StyledText`, and `SelectableText`.
- [x] Confirm the matrix names every rejection reason that can surface as `text_reflow`.
- [x] Decide whether the matrix belongs in this workstream only or in a longer-lived runtime
  contract doc.

Decision: keep the matrix in this workstream for now. Promote it to a longer-lived runtime contract
doc only if a future behavior change expands text clean-geometry eligibility.

## T2 - Diagnostics Shape

- [x] Check whether current clean-geometry rejection stats expose enough text metadata for future
  line-break proofs.
- [x] If missing, add source-stable debug fields without changing runtime text behavior.
- [x] Capture a UI Gallery text-measure-overlay resize bundle proving the additive `detail` field
  reaches shareable diagnostics artifacts.

Decision: the existing diagnostics path already exposed rejection `reason`, rejected `node`,
`element_kind`, and `element_path`, but all text sub-causes collapsed into `text_reflow`. Additive
`detail` now distinguishes text sub-causes such as `text_wrap_not_none` without changing layout
decisions.

## T3 - Optional Wrapped-Text Proof

- [x] Decide not to prototype wrapped-text clean geometry in this boundary lane because no behavior
  change is being shipped here.
- [x] Record that any future wrapped-text lane must start only after fresh evidence shows wrapped
  text is a material perf owner.
- [x] Require unchanged line count, measured size, glyph cluster positions, and paint-relevant
  metrics before any wrapped text can skip authoritative layout.
- [x] Keep shadcn `CardDescription` as `TextWrap::Word`; recipe authoring is not the optimization
  lever.

## T4 - Perf Recheck

- [x] Record that no behavior change landed in this lane, so a perf recheck is not required for
  closeout.
- [x] Require any future behavior-changing wrapped-text lane to rerun the UI Gallery resize-jitter
  perf repro from the parent performance lane and compare the remaining `text_reflow / Text`
  contribution.
