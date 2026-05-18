# WGPU Test Support Dead Code Prune v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Split Support Entry Points

- [x] WTSD-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/support,crates/fret-render-wgpu/tests/*]
  Goal: Remove the final test-support `dead_code` allowance by compiling only the helpers each test
  binary uses.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; residual dead-code scan.
  Evidence: readback-only tests use `support/readback.rs`, explicit-format tests use
  `support/render_format.rs`, and default scene tests keep `support/mod.rs`.
  Status: Done on 2026-05-18.

## M1 - Representative Support Gates

- [x] WTSD-020 [owner=codex] [deps=WTSD-010] [scope=crates/fret-render-wgpu/tests]
  Goal: Prove all three support entry points still work.
  Validation: targeted nextest for default scene render, explicit format render, and readback-only
  paths.
  Evidence: `gpu_path_fill_rules_distinguish_overlapping_winding_regions`,
  `gpu_composite_group_add_is_scissored_and_additive`, and
  `gpu_non_srgb_output_applies_explicit_srgb_transfer` passed.
  Status: Done on 2026-05-18.

## M2 - Closeout

- [x] WTSD-030 [owner=codex] [deps=WTSD-020] [scope=docs/workstreams/wgpu-test-support-dead-code-prune-v1,docs/workstreams/README.md]
  Goal: Record the integration-test support invariant and close the follow-on.
  Validation: `python tools/check_workstream_catalog.py`; `python -m json.tool docs/workstreams/wgpu-test-support-dead-code-prune-v1/WORKSTREAM.json`; `git diff --check`.
  Evidence: closeout audit records that no `dead_code` allowance remains in `fret-render-wgpu`.
  Status: Done on 2026-05-18.
