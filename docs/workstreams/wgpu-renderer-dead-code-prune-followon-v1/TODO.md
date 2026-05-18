# WGPU Renderer Dead Code Prune Follow-on v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Production Dead Code Prune

- [x] WRDP-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src]
  Goal: Remove production `dead_code` suppressions with clear no-caller/no-reader evidence.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`.
  Evidence: `BindGroupCaches::invalidate_all`, `TextSystem::prepare_input`,
  `subpixel_mask_to_alpha`, and returned `DownsampleHalfQuarter.half_size` are removed; called
  color-matrix/alpha-threshold helpers no longer carry stale suppressions.
  Status: Done on 2026-05-18.

## M1 - Focused Regression Gates

- [x] WRDP-020 [owner=codex] [deps=WRDP-010] [scope=crates/fret-render-wgpu/src/renderer/render_plan.rs,crates/fret-render-wgpu/src/text]
  Goal: Prove the edited text and render-plan areas still compile and retain representative behavior.
  Validation: `cargo nextest run -p fret-render-wgpu --locked downsample_half_quarter_helper_emits_two_passes`; `cargo nextest run -p fret-render-wgpu --locked paint_span_for_text_range_is_directional_across_span_boundary`.
  Evidence: targeted nextest runs passed.
  Status: Done on 2026-05-18.

## M2 - Closeout

- [x] WRDP-030 [owner=codex] [deps=WRDP-020] [scope=docs/workstreams/wgpu-renderer-dead-code-prune-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence, explicitly preserve test-only suppressions, and close the lane.
  Validation: `python tools/check_workstream_catalog.py`; `python -m json.tool docs/workstreams/wgpu-renderer-dead-code-prune-followon-v1/WORKSTREAM.json`; `git diff --check`.
  Evidence: closeout audit names deleted residue and remaining test-only allowances.
  Status: Done on 2026-05-18.
