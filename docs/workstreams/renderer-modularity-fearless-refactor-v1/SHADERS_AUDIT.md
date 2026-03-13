# Renderer Modularity (Fearless Refactor v1) — Shaders Audit

Status: Closed for v1

Last updated: 2026-03-13

## Scope

This audit answers the remaining shader-modularity question for v1:

- does `crates/fret-render-wgpu/src/renderer/shaders.rs` still need ownership-oriented splitting,
  or has it already reached the intended boring shape?

## Findings

### 1. `shaders.rs` is now an index/assembly file, not an oversized ownership hotspot

Current shape:

- the file is now 331 lines instead of the previous multi-thousand-line inline-WGSL store
- all large WGSL bodies live under `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*.wgsl`
- the remaining Rust file only does three things:
  - indexes `include_str!` WGSL sources
  - assembles the small multi-part masked/envelope shader variants
  - exposes a tiny local naga guard for `PATH_SHADER`

Evidence:

- `crates/fret-render-wgpu/src/renderer/shaders.rs`
- `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/`
- `crates/fret-render-wgpu/src/renderer/tests.rs`

Assessment:

- further splitting `shaders.rs` into one Rust module per shader family would not create a new
  ownership boundary; it would only spread one index/assembly pattern across more files
- the real reviewable units are already the WGSL files under `pipelines/wgsl/`
- the remaining assembly helpers are intentionally local because they share the same
  `CLIP_SDF_CORE_WGSL` / envelope stitching pattern

Decision:

- close `RMFR-shaders-050` as done
- close `RMFR-shaders-051` as done
- keep `shaders.rs` as the v1 shader index/assembly surface
- only revisit if a future shader family needs its own Rust-side stateful code generation or
  nontrivial compile-time policy beyond string assembly

### 2. WGSL validation coverage is aligned with the reorganized source layout

Current shape:

- `crates/fret-render-wgpu/src/renderer/tests.rs` parses and validates the full in-tree shader set,
  including assembled masked variants and custom-effect envelopes
- `crates/fret-render-wgpu/src/renderer/shaders.rs` keeps a small local naga validation guard for
  `PATH_SHADER`
- a conditional wasm/browser WebGPU Tint guard still exists in
  `crates/fret-render-wgpu/src/renderer/tests.rs`

Evidence:

- `crates/fret-render-wgpu/src/renderer/tests.rs`
- `crates/fret-render-wgpu/src/renderer/shaders.rs`

Assessment:

- native in-tree WGSL parse + naga validation coverage is already sufficient to close the v1 source
  reorganization task
- the conditional wasm/browser guard remains useful as future drift detection, but it does not
  block closing the current alignment task

Decision:

- close `RMFR-shaders-052` as done for v1

## Closure Summary

For v1:

- `shaders.rs` stays as a small shader index/assembly file
- `pipelines/wgsl/*.wgsl` stays as the reviewable source-of-truth for shader bodies
- shader modularity work no longer needs ownership-driven file splitting
