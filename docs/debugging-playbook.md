# Debugging Playbook (Passes, Layout, Screenshots)

This document is a living, practical checklist for debugging Fret. It focuses on **repeatable** workflows
over ad-hoc guessing.

All debug artifacts (captures, dumps, temporary outputs) should live under `.fret/` to avoid accidental
commits.

## 0) UI diagnostics bundles (AI-friendly repro units)

For UI debugging (input/focus/overlays) and for agent-friendly triage, prefer collecting a **diagnostics bundle**
instead of ad-hoc logs. Bundles capture:

- per-frame UI snapshots (including semantics),
- input events, hit test info, layer roots,
- optional GPU-readback screenshots (for visual overlay debugging).

Core docs:

- Bundles + scripts: `docs/ui-diagnostics-and-scripted-tests.md`
- Interactive inspect workflow: `docs/debugging-ui-with-inspector-and-scripts.md`

Quick workflow (recommended default paths under `.fret/`):

```powershell
$env:FRET_DIAG=1
$env:FRET_DIAG_DIR=".fret\\diag"

# Optional: enable screenshot protocol (required for `capture_screenshot` if the app is started outside `fretboard --launch`).
$env:FRET_DIAG_SCREENSHOTS=1

# Run a deterministic scripted repro and auto-pack a shareable zip.
cargo run -p fretboard -- diag repro tools/diag-scripts/ui-gallery-intro-idle-screenshot.json `
  --launch -- cargo run -p fret-ui-gallery --release

# Equivalent (lower-level) form:
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json `
  --pack --include-all `
  --launch -- cargo run -p fret-ui-gallery --release
```

Notes:

- `fretboard diag inspect on` enables a GPUI/Zed-style picker overlay and writes `pick.result.json` for stable selectors.
- `fretboard diag pack --include-all` and `diag run --pack --include-all` produce a `.zip` that the offline viewer can open (`tools/fret-bundle-viewer`).
- `diag repro` writes `repro.zip` and `repro.summary.json` into `FRET_DIAG_DIR` (useful as a single “attach this” artifact).
  - When running a suite, `repro.zip` includes multiple bundles under stable prefixes, plus script sources under `_root/scripts/`.

Framework consistency checks (automation-friendly):

- **Stale paint detection**: fails when a semantics node moves but the scene fingerprint does not change (common symptom:
  “UI updated but pixels didn’t repaint”).
- **Stale scene detection**: fails when a semantics node’s label/value changes (or moves) but the scene fingerprint does
  not change (common symptom: “search results changed but text didn’t repaint / disappeared”).
- **Semantics repaint detection**: fails when the bundle’s `semantics_fingerprint` changes but `scene_fingerprint` does
  not (a coarse “something semantic changed but we didn’t repaint” signal).
- **Pixels changed detection**: fails when a screenshot-backed region hash inside the target semantics bounds does not
  change across captures (tooling: `--check-pixels-changed <test_id>`; evidence: `check.pixels_changed.json`).

Example:

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json `
  --check-stale-paint ui-gallery-nav-intro `
  --launch -- cargo run -p fret-ui-gallery --release
```

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-code-view-scroll-refresh.json `
  --check-stale-scene <test_id> `
  --launch -- cargo run -p fret-ui-gallery --release
```

Screenshot-backed example (requires `capture_screenshot` steps):

```powershell
cargo run -p fretboard -- diag repro tools/diag-scripts/ui-gallery-code-view-scroll-refresh-pixels-changed.json `
  --check-pixels-changed ui-gallery-code-view-root `
  --launch -- cargo run -p fret-ui-gallery --release
```

## 1) GPU / renderer: debug specific passes

### 1.1 Capture a frame (RenderDoc)

On Windows, prefer Vulkan for capture reliability.

```powershell
$env:FRET_WGPU_BACKEND="vulkan"
$env:FRET_RENDERDOC=1
$env:FRET_RENDERDOC_DLL="C:\\Users\\Frankorz\\scoop\\apps\\renderdoc\\current\\renderdoc.dll"
# Optional:
$env:FRET_RENDERDOC_AUTOCAPTURE=1
# Optional: capture a later, more representative frame (overrides `FRET_RENDERDOC_AUTOCAPTURE`):
# $env:FRET_RENDERDOC_AUTOCAPTURE_AFTER_FRAMES=60
# Optional:
$env:FRET_RENDERDOC_CAPTURE_DIR=".fret\\renderdoc"

cargo run -p fret-demo --bin fret-demo -- effects_demo
```

Captures are written under `.fret/renderdoc/` (or `FRET_RENDERDOC_CAPTURE_DIR`).

### 1.1.1 Dump the CPU RenderPlan (no RenderDoc required)

If you need to validate pass structure/scissors/origins without a GPU capture, enable the CPU-side plan dump:

```powershell
$env:FRET_RENDERPLAN_DUMP=1
# Optional:
# $env:FRET_RENDERPLAN_DUMP_FRAME=60
# $env:FRET_RENDERPLAN_DUMP_AFTER_FRAMES=60
# $env:FRET_RENDERPLAN_DUMP_EVERY=60
# $env:FRET_RENDERPLAN_DUMP_DIR=".fret\\renderplan"

cargo run -p fret-demo --bin fret-demo -- effects_demo
```

The renderer writes JSON files under `.fret/renderplan/` (or `FRET_RENDERPLAN_DUMP_DIR`).

### 1.2 Inspect a pass (scriptable)

Use `fret-renderdoc` to search marker substrings, export outputs, and dump key buffers to JSON:

```powershell
cargo run -p fret-renderdoc -- dump `
  --renderdoc-dir "C:\\Users\\Frankorz\\scoop\\apps\\renderdoc\\current" `
  --capture ".fret\\renderdoc\\fret_1.rdc" `
  --marker "fret clip mask pass"
```

See `docs/renderdoc-inspection.md` for what fields to validate for common passes.

Tip: for a quick "frame health" breakdown (which marker paths dominate drawcalls), run with an empty marker and a
high `--max-results`, then inspect `result.summary.*` in the JSON response (see `docs/renderdoc-inspection.md`).

Tip: if you're debugging pixelate/blur effects, start with:

```powershell
cargo run -p fret-renderdoc -- dump `
  --renderdoc-dir "C:\\Users\\Frankorz\\scoop\\apps\\renderdoc\\current" `
  --capture ".fret\\renderdoc\\fret_capture.rdc" `
  --marker "nearest" `
  --selection all `
  --no-outputs-png
```

### 1.3 What “correct” usually looks like

Common invariants we validate from captures:

- `raster_state.scissors[0]` matches the intended effect bounds (or mask target rect).
- For viewport-scoped masks: `mask_viewport_origin/size` matches the effect viewport rect (not the full window).
- For scale/pixelate chains: scissored fullscreen passes must be **origin-aware** (avoid anchoring to `(0,0)`).

### 1.4 Debugging transforms / “matrix” issues

In Fret, the most common “matrix looks wrong” bugs are not about layout, but about **render transforms**
(paint + hit testing + pointer coordinates).

Contracts:

- RenderTransform semantics: `docs/adr/0083-render-transform-hit-testing.md`
- Scene transform + clip composition: `docs/adr/0078-scene-transform-and-clip-composition.md`

Practical checklist:

1. Confirm whether the issue is **layout** (bounds wrong) or **visual** (bounds right, but rendered output shifted/rotated).
2. For visual issues, verify the transform stack in the `SceneOp` stream (Push/Pop pairs).
3. If the bug crosses effect boundaries, validate in RenderDoc:
   - scissor/viewport state for the pass,
   - clip stack head/count and mask viewport origin/size (for masked writebacks),
   - output target(s) at each step (export PNGs).

### 1.5 Debugging streaming image updates (video frames)

When debugging `Effect::ImageUpdate*` ingestion (ADR 0121 / ADR 0126), prefer collecting *both*:

- per-frame counters via `fret_core::StreamingUploadPerfSnapshot`, and
- (optional) runner debug logs when drops/delays happen.

Useful env vars:

```powershell
# Update `StreamingUploadPerfSnapshot` and log budget/drops (when relevant).
$env:FRET_STREAMING_DEBUG="1"

# Optional: debug override to try the NV12 GPU path (experimental; falls back to CPU).
$env:FRET_STREAMING_GPU_YUV="1"

# Demo-only helpers (apps/fret-examples streaming_*_demo.rs).
$env:FRET_DEMO_STREAMING_PERF_EVERY="60"
$env:FRET_DEMO_AUTO_EXIT_FRAMES="240"
```

Run a minimal repro demo:

```powershell
cargo run -p fret-demo --bin streaming_nv12_demo
# or:
cargo run -p fret-demo --bin streaming_i420_demo
```

Notes on the NV12 GPU path (`FRET_STREAMING_GPU_YUV=1`):

- Currently only accelerates `Effect::ImageUpdateNv12` into `Rgba8UnormSrgb` image storage (sRGB) and forces
  `AlphaMode::Opaque` for the target image.
- `StreamingUploadPerfSnapshot.yuv_convert_us` measures CPU-side work (plane repack + command encoding), not GPU time.
- `StreamingUploadPerfSnapshot.upload_bytes_budgeted` reflects the conservative estimate used for budgets, while
  `upload_bytes_applied` reflects actual CPU->GPU uploads performed by the applied path.
- A quick sanity check is that `streaming_nv12_demo` should show a significantly smaller `yuv_us` vs CPU fallback.

Notes on capability gating (ADR 0124):

- The runner publishes a per-session capability snapshot as an app global: `fret_render::RendererCapabilities`.
- NV12 GPU conversion requires both:
  - `RendererCapabilities.streaming_images.nv12_gpu_convert == true` (supported), and
  - an enable switch (either `WinitRunnerConfig.streaming_nv12_gpu_convert_enabled = true` or `FRET_STREAMING_GPU_YUV=1`).

If you need structured logs from the runner:

```powershell
$env:RUST_LOG="fret_launch=debug"
```

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

### 2.1 Dump Taffy layout trees (layout engine v2)

When debugging layout engine v2 (enabled by default in this repository), it is often faster to inspect the solved Taffy tree(s) than to
guess which wrapper collapsed. Fret can emit JSON dumps under `.fret/`.

```powershell
# Enable dumps.
$env:FRET_TAFFY_DUMP=1
# Optional: write to a dedicated directory.
$env:FRET_TAFFY_DUMP_DIR=".fret\\taffy-dumps"
# Optional: cap output count to avoid spam.
$env:FRET_TAFFY_DUMP_MAX=30
# Optional: only dump roots whose NodeId string contains this substring.
$env:FRET_TAFFY_DUMP_ROOT="NodeId(46"
# Optional: only dump roots whose element debug label contains this substring.
# Tip: if you wrap a region in a `SemanticsProps { label: Some("Golden:..."), .. }`, you can
# filter by that label. Note: the label match is applied to the root subtree, so you can match a
# nested `SemanticsProps.label` even if the window root is an internal wrapper.
$env:FRET_TAFFY_DUMP_ROOT_LABEL="Golden:input-with-label"

# Run a repro (example):
cargo run -p fret-demo --bin todo_demo
```

Notes:

- Dumps include **window roots** and **viewport roots** (e.g. scroll content) as separate files.
- Each node entry includes `node/parent/children`, `local_rect/abs_rect`, the computed Taffy `style`,
  and a debug `label` derived from the element instance.
- Prefer filtering by a stable semantics label when possible:
  - Wrap the root you care about with `SemanticsProps { label: Some("Golden:..."), .. }`.
  - Set `FRET_TAFFY_DUMP_ROOT_LABEL="Golden:..."` to dump the first matching node’s subtree,
    without chasing unstable `NodeId(...)` values across runs.

### 2.1.1 Detect widget-local layout engine fallback solves

Layout engine v2 is designed around a window-scoped pipeline (request/build → solve → apply). If a
widget can’t consume already-solved engine child rects, it may trigger a “widget-local” fallback
solve to keep the UI functional. This is useful as a compatibility escape hatch, but it is also a
signal that the layout tree is drifting from the intended contracts.

To surface these issues:

```powershell
# Panic on the first fallback solve (useful for CI and tightening invariants).
$env:FRET_LAYOUT_FORBID_WIDGET_FALLBACK_SOLVES=1

# Or: keep running but log each fallback solve with node + element labels.
$env:FRET_LAYOUT_TRACE_WIDGET_FALLBACK_SOLVES=1

# Run tests (example):
cargo nextest run -p fret-ui -p fret-docking
```

In debug builds with `UiTree::set_debug_enabled(true)`, the frame stats include a counter:
`UiDebugFrameStats.layout_engine_widget_fallback_solves`.

Tip: when using `fret-bootstrap`'s UI diagnostics, setting `FRET_DIAG=1` will also enable UI frame stats
(same effect as `FRET_UI_DEBUG_STATS=1`) so bundles include `UiDebugFrameStats` counters.

Example (todo demo):

```powershell
$env:FRET_TAFFY_DUMP=1
$env:FRET_TAFFY_DUMP_ONCE=1
$env:FRET_TAFFY_DUMP_DIR=".fret\\taffy-dumps"
$env:FRET_TAFFY_DUMP_ROOT_LABEL="Debug:todo-demo:page"
cargo run -p fret-demo --bin todo_demo
```

### 2.2 Prefer a unit test when possible

If the bug is deterministic (no timing/input dependency), it is usually faster to reproduce it as a test:

- Declarative geometry tests live under: `crates/fret-ui/src/declarative/tests/`
- Text layout tests live under: `crates/fret-ui/src/text_area/tests.rs`

Typical pattern:

1. Build a small UI tree.
2. Call `layout_all(...)` with a fixed window bounds + scale factor.
3. Assert the computed bounds/visual bounds for the relevant node(s).

### 2.3 Debug “visual bounds vs layout bounds”

For overlay anchoring and transformed widgets, always distinguish:

- layout bounds (most recently recorded): `bounds_for_element(...)`
- visual bounds (post-transform AABB, most recently recorded): `visual_bounds_for_element(...)`

Reference tests (anchored overlays): `crates/fret-ui/src/declarative/tests/anchored.rs`.

## 3) Screenshots / readback (deterministic pixels)

For deterministic pixel validation, prefer existing GPU conformance tests (readback-based):

- `cargo nextest run -p fret-render`

These tests are designed to catch:

- ordering leaks across effect boundaries,
- scissor/clip regressions,
- mask mapping errors.

### 3.1 Where readback code lives

Renderer readback helpers are implemented inside the conformance tests:

- `crates/fret-render/tests/*_conformance.rs` (look for `render_and_readback(...)`)

If you need a new regression test, prefer adding a minimal scene + asserting pixel properties over adding
new ad-hoc debug output.

### 3.2 Optional: write readback bytes to a PNG (debug-only)

Sometimes it is useful to inspect the readback pixels directly in an image viewer. This should remain a
**local debugging workflow**:

- write outputs under `.fret/debug/` (ignored by git by default),
- avoid writing files in CI (gate behind a local env var),
- do not treat disk output as a stable contract (tests should assert on pixels, not rely on files).

Most readback buffers are padded to a 256-byte row alignment (`bytes_per_row`), so you usually need to
repack the rows before saving.

Example helper (requires a local dev-dependency on `image`):

```rust
fn write_rgba8_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    bytes_per_row: usize,
    data: &[u8],
) -> anyhow::Result<()> {
    use anyhow::Context as _;

    let tight_bpr = width as usize * 4;
    anyhow::ensure!(bytes_per_row >= tight_bpr, "bytes_per_row < width*4");
    anyhow::ensure!(data.len() >= bytes_per_row * height as usize, "buffer too small");

    let mut tight = vec![0u8; tight_bpr * height as usize];
    for y in 0..height as usize {
        let src = &data[y * bytes_per_row..y * bytes_per_row + tight_bpr];
        let dst = &mut tight[y * tight_bpr..y * tight_bpr + tight_bpr];
        dst.copy_from_slice(src);
    }

    let img = image::RgbaImage::from_raw(width, height, tight)
        .context("construct RgbaImage")?;
    img.save(path).context("save png")?;
    Ok(())
}
```

Notes:

- If the capture target is `*Srgb`, the bytes are already in sRGB space; save them as-is.
- If you read back from a linear target, you may need to apply a transfer function before saving for
  visual inspection.

For app-facing screenshots/recording, the contract is defined by:

- `docs/adr/0122-offscreen-rendering-frame-capture-and-readback.md`

Note: encoding (PNG/MP4) is app-owned; the framework’s responsibility is a portable readback mechanism and
bounded backpressure.

## 4) CPU tracing / observability

When the bug is “why did this frame render”, use structured tracing:

- Observability contract: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- Frame identity/scheduling: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`

If the symptom is “dragging the window feels laggy / delayed”, first rule out debug overhead:

- Ensure heavy debug dumps are disabled (`FRET_TAFFY_DUMP` writes JSON to disk and will stutter).
- Prefer `--release` to avoid debug build overhead.
- Reduce log volume (e.g. `RUST_LOG=warn`) to avoid per-frame stdout overhead.

To locate the bottleneck, enable frame hitch logging (writes only when a frame exceeds a threshold):

```powershell
$env:FRET_FRAME_HITCH_LOG=1
# Default is 24ms; adjust as needed for your monitor / expectation.
$env:FRET_FRAME_HITCH_MS=24
cargo run -p fret-demo --bin todo_demo
```

The log is written to `.fret/frame_hitches.log` (and also mirrored under the system temp dir).
Each entry includes the breakdown of `view` / `overlay` / `layout` / `paint`, plus `scene_ops`.

If `.fret/frame_hitches.log` stays quiet but the app still feels laggy, the hitch is likely outside
the UI tree work (e.g. surface acquire/present or GPU work). Enable redraw hitch logging:

```powershell
$env:FRET_REDRAW_HITCH_LOG=1
$env:FRET_REDRAW_HITCH_MS=24
cargo run -p fret-demo --bin todo_demo
```

The log is written to `.fret/redraw_hitches.log` and includes `prepare` / `render` / `record` /
`present` timings plus any surface error.

Recommended approach:

1. Turn on tracing for the smallest set of crates relevant to the bug.
2. Correlate the frame reason (input/timer/animation) with renderer passes (RenderDoc markers).

### 4.1 Tracy (timeline view via `tracing`)

Fret can stream `tracing` spans/events into the Tracy profiler for a **timeline view** (frame phases,
threads, and nested spans).

See also: `docs/tracy.md`.

Notes:

- This is currently a **native-only** workflow (not `wasm32`).
- Tracy complements (not replaces) diagnostics bundles:
  - use `fretboard diag perf` / `diag stats --sort time` to find the slowest frames,
  - use Tracy to inspect **what ran** during those frames.

#### 4.1.1 Enable Tracy

1. Start Tracy Profiler (the UI) and wait for a client connection.
2. Run your app with the `fret-bootstrap/tracy` feature and the `FRET_TRACY=1` env var:

```powershell
$env:FRET_TRACY=1

# Optional: include cache-root spans inside `fret-ui` (these are `TRACE` level).
$env:RUST_LOG="info,fret_ui=trace"

cargo run -p fret-ui-gallery --release --features fret-bootstrap/tracy
```

If you don't enable `fret_ui=trace`, you'll still see the higher-level frame spans, but not the
per-cache-root spans.
