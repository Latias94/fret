# Debugging Playbook (Passes, Layout, Screenshots)

This document is a living, practical checklist for debugging Fret. It focuses on **repeatable** workflows
over ad-hoc guessing.

## 1) GPU / renderer: debug specific passes

### 1.1 Capture a frame (RenderDoc)

On Windows, prefer Vulkan for capture reliability.

```powershell
$env:FRET_WGPU_BACKEND="vulkan"
$env:FRET_RENDERDOC=1
$env:FRET_RENDERDOC_DLL="C:\\Users\\Frankorz\\scoop\\apps\\renderdoc\\current\\renderdoc.dll"
# Optional:
$env:FRET_RENDERDOC_AUTOCAPTURE=1
# Optional:
$env:FRET_RENDERDOC_CAPTURE_DIR=".fret\\renderdoc"

cargo run -p fret-demo --bin fret-demo -- effects_demo
```

Captures are written under `.fret/renderdoc/` (or `FRET_RENDERDOC_CAPTURE_DIR`).

### 1.2 Inspect a pass (scriptable)

Use `fret-renderdoc` to search marker substrings, export outputs, and dump key buffers to JSON:

```powershell
cargo run -p fret-renderdoc -- dump `
  --renderdoc-dir "C:\\Users\\Frankorz\\scoop\\apps\\renderdoc\\current" `
  --capture ".fret\\renderdoc\\fret_1.rdc" `
  --marker "fret clip mask pass"
```

See `docs/renderdoc-inspection.md` for what fields to validate for common passes.

### 1.3 What “correct” usually looks like

Common invariants we validate from captures:

- `raster_state.scissors[0]` matches the intended effect bounds (or mask target rect).
- For viewport-scoped masks: `mask_viewport_origin/size` matches the effect viewport rect (not the full window).
- For scale/pixelate chains: scissored fullscreen passes must be **origin-aware** (avoid anchoring to `(0,0)`).

## 2) Layout debugging

Layout issues are often “correct math, wrong contract”. Prefer to debug at contract boundaries:

- **UI tree layout entry point:** `crates/fret-ui/src/frame_cx.rs` (`layout_all` call site).
- **Widget layout helpers:** `crates/fret-ui/src/widget.rs` (`layout_in`, `layout_engine_child_bounds`).
- **Declarative host layout:** `crates/fret-ui/src/declarative/host_widget/layout.rs`.

Practical workflow:

1. Reduce to the smallest demo that reproduces the issue (`apps/fret-examples`).
2. Verify the bounds handed to the root (window size, scale factor).
3. Verify child bounds propagation (especially when using absolute positioning helpers).
4. If the issue is about what the user “sees”, validate visual bounds vs layout bounds (ADR 0083).

## 3) Screenshots / readback (deterministic pixels)

For deterministic pixel validation, prefer existing GPU conformance tests (readback-based):

- `cargo nextest run -p fret-render`

These tests are designed to catch:

- ordering leaks across effect boundaries,
- scissor/clip regressions,
- mask mapping errors.

For app-facing screenshots/recording, the contract is defined by:

- `docs/adr/0122-offscreen-rendering-frame-capture-and-readback.md`

Note: encoding (PNG/MP4) is app-owned; the framework’s responsibility is a portable readback mechanism and
bounded backpressure.

## 4) CPU tracing / observability

When the bug is “why did this frame render”, use structured tracing:

- Observability contract: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- Frame identity/scheduling: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`

Recommended approach:

1. Turn on tracing for the smallest set of crates relevant to the bug.
2. Correlate the frame reason (input/timer/animation) with renderer passes (RenderDoc markers).

