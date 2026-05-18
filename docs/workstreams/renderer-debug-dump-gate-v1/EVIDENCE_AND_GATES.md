# Renderer Debug Dump Gate v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Gate Set

```bash
cargo fmt --package fret-render-wgpu --check
cargo check -p fret-render-wgpu --locked --tests -j 1
cargo nextest run -p fret-render-wgpu --locked dump_frame_gate render_text_dump_state_clear_scratch_keeps_capacity
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/renderer-debug-dump-gate-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu --check`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked dump_frame_gate render_text_dump_state_clear_scratch_keeps_capacity`
  - Result: nextest run ID `f7c38dcf-a42f-4ea6-bb3b-c667ddfc671b`; 6 tests run, 6 passed, 285 skipped.
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/renderer-debug-dump-gate-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Evidence Anchors

- `crates/fret-render-wgpu/src/renderer/debug_dump_gate.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_dump_emit.rs`
- `crates/fret-render-wgpu/src/renderer/render_text_dump.rs`
- `crates/fret-render-wgpu/src/renderer/mod.rs`
- `docs/workstreams/renderer-debug-dump-gate-v1/CLOSEOUT_AUDIT_2026-05-18.md`
