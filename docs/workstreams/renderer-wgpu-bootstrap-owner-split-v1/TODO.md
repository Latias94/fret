# Renderer WGPU Bootstrap Owner Split v1 — TODO

Status: Active
Last updated: 2026-05-17

## M0 — Bootstrap Owner Extraction

- [x] RWBO-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/lib.rs,crates/fret-render-wgpu/src/context.rs,crates/fret-render/src/lib.rs,crates/fret-render/tests/facade_surface_snapshot.rs]
  Goal: Move `WgpuContext`, init diagnostics, adapter snapshot construction, backend override parsing, and adapter validation out of the backend crate root into a private context/bootstrap owner module.
  Validation: `cargo test -p fret-render --locked --test facade_surface_snapshot -j 1`; `cargo test -p fret-render-wgpu --locked --lib parse_wgpu_backends -j 1`; `cargo check -p fret-render-wgpu --locked --tests -j 1`.
  Evidence: `lib.rs` becomes export/module glue; public type paths remain stable through re-exports.
  Handoff: This is an ownership/locality slice only. Do not change GPU selection policy, environment variable names, diagnostics schema, or facade buckets.
  Status: Done on 2026-05-17. `context.rs` now owns the bootstrap implementation and `lib.rs` keeps stable public re-exports.

## M1 — Closeout

- [ ] RWBO-020 [owner=planner] [deps=RWBO-010] [scope=docs/workstreams/renderer-wgpu-bootstrap-owner-split-v1]
  Goal: Close this narrow lane after the owner split or record a smaller follow-on only if the first slice exposes a real second bootstrap owner.
  Validation: `WORKSTREAM.json`, `TODO.md`, and `EVIDENCE_AND_GATES.md` agree.
  Evidence: closeout note or updated status.
