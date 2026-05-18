# Renderer Debug Dump Gate v1 - Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped

- Added `debug_dump_gate` as the native renderer-internal owner for debug dump frame gating.
- Removed duplicated env frame gate parsing from render-plan and text dump code.
- Kept dump-specific env prefixes, default directories, filenames, and JSON schemas unchanged.
- Removed unreachable wasm branches from native-only dump modules.

## Verification

- `cargo fmt --package fret-render-wgpu --check`
- `cargo check -p fret-render-wgpu --locked --tests -j 1`
- `cargo nextest run -p fret-render-wgpu --locked dump_frame_gate render_text_dump_state_clear_scratch_keeps_capacity`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/renderer-debug-dump-gate-v1/WORKSTREAM.json`
- `git diff --check`

## Residual Risk

The gate tests cover the shared frame selection semantics. They do not write real dump files, so
filesystem errors remain intentionally best-effort as before.
