# Renderer Debug Dump Gate v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Centralize Dump Gate Mechanism

- [x] RDDG-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/renderer]
  Goal: Move duplicated render-plan/text dump frame gate and directory selection logic into a
  shared renderer-internal module without changing dump schemas, filenames, or env var names.
  Validation: `cargo nextest run -p fret-render-wgpu --locked dump_frame_gate render_text_dump_state_clear_scratch_keeps_capacity`.
  Evidence: shared gate tests passed and the existing text dump scratch test still passed.
  Status: Done on 2026-05-18.

## M1 - Compile And Closeout

- [x] RDDG-020 [owner=codex] [deps=RDDG-010] [scope=crates/fret-render-wgpu,docs/workstreams/renderer-debug-dump-gate-v1]
  Goal: Prove the renderer crate still compiles with tests and record the closed follow-on.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_workstream_catalog.py`; `python -m json.tool docs/workstreams/renderer-debug-dump-gate-v1/WORKSTREAM.json`; `git diff --check`.
  Evidence: targeted Rust and workstream catalog gates passed.
  Status: Done on 2026-05-18.
