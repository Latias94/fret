# WGPU Host Topology Smoke Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The host-provided GPU topology smoke test duplicated RGBA8 readback and pixel helpers that are now
owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owner:

- `crates/fret-render-wgpu/tests/host_provided_gpu_topology_smoke.rs`

Explicit non-scope:

- Direct adapter/device/queue request and renderer construction remain local because they are the
  core smoke-test contract.
- The test does not use `tests/support::render_scene_rgba8` because that helper requires
  `WgpuContext` and would erase the host-provided topology proof.
- Vulkan and MSAA tests stay out of scope for this lane.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test host_provided_gpu_topology_smoke -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-host-topology-smoke-harness-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test host_provided_gpu_topology_smoke -j 1`
  - Result: nextest run ID `c47df71b-dc9e-4f30-b2ac-a4b4b72e59cb`; 1 test run, 1 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 404 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-host-topology-smoke-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-host-topology-smoke-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/host_provided_gpu_topology_smoke.rs`
