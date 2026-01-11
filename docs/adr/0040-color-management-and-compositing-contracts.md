# ADR 0040: Color Management and Compositing Contracts (Linear Compositor, Viewport Metadata)

Status: Accepted

## Context

Fret targets engine editors with:

- engine-rendered viewports embedded in UI (ADR 0007),
- UI overlays (gizmos, selection outlines, dock chrome) composited over those viewports,
- multiple OS windows and surfaces (ADR 0017),
- future wasm/WebGPU portability.

If we do not lock color/compositing rules early, we will later be forced to retrofit:

- inconsistent blending between UI and engine viewports,
- “washed out” or “too dark” colors due to double sRGB conversions,
- HDR/linear confusion that makes debugging GPU output painful.

## Decision

### 1) Compositing space is linear

All blending/compositing in the renderer is done in **linear color space**:

- UI primitives (quads, shadows) are shaded in linear.
- Text glyph coverage is applied in linear (after glyph sampling).
- Viewport textures are sampled into linear and composited in linear.

### 2) UI color tokens are authored as sRGB, converted in the shader

UI theme/style colors are authored as sRGB values (human-friendly).

The renderer is responsible for converting them to linear for shading/compositing.

This keeps authoring ergonomics good while keeping the GPU pipeline correct.

Contract note:

- `fret-core::Color` values emitted into `SceneOp` are treated as **linear** values by the renderer.
  If UI/theme authoring uses sRGB, conversion must happen before the color reaches the renderer (CPU-side or shader-side).

### 3) Surface formats are treated as display-referred outputs

For desktop surfaces:

- Prefer an sRGB swapchain surface format when available (`*Srgb`).
- The renderer writes linear outputs; the GPU performs the linear→sRGB conversion as part of rendering to an sRGB target.

If a non-sRGB surface format is used, the renderer must apply the appropriate transfer function explicitly.

### 4) Engine viewport targets must declare their encoding via renderer-owned metadata

Because `RenderTargetId` resolution is renderer-owned and wgpu-facing (ADR 0007), the renderer’s target registry
must record **how a target should be sampled**:

- `encoding = Srgb`: sample from an sRGB texture view (`*Srgb`) so sampling yields linear.
- `encoding = Linear`: sample from a linear view/format (e.g. float or unorm linear).
- `encoding = HdrDisplayReferred`: reserved for future HDR paths (PQ/HLG), not implemented in P0.

This metadata lives in the renderer registry (wgpu-side), not in `fret-core`.

### 5) Viewport contract: display-referred by default (no tonemapper in Fret)

Fret is a UI framework, not an engine renderer.

Therefore, the mainline contract is:

- engine viewports registered for UI display are **display-referred** (already tonemapped to the intended output range),
- Fret does not perform tonemapping on viewport targets in P0.

If an engine wants to preview HDR/linear buffers, it should provide a debug “viewer” target as another render target.

### 6) Alpha rules are explicit (premultiplied)

All UI blending uses premultiplied alpha (consistent with ADR 0002).

Viewport targets should be treated as opaque by default; if alpha is meaningful, the render target registry must
explicitly mark it as premultiplied vs opaque.

### 7) Authoring ergonomics: make premultiplied alpha hard to misuse

Premultiplied alpha is easy to misuse at call sites (straight RGBA values produce “too bright” translucent fills).

To reduce recurring bugs in demos and component ecosystems, producers should adopt a stable authoring pattern:

- Provide a helper constructor that takes straight RGBA and returns premultiplied linear RGBA, e.g.
  `Color::from_rgba_premul(r, g, b, a)` (name is illustrative).
- In debug builds, validate that colors emitted into `SceneOp` are plausibly premultiplied:
  - if `a < 1` and any of `r/g/b > a + eps`, emit a diagnostic (warn or deny depending on policy).

This does not change renderer semantics; it improves correctness and reduces “mysterious compositing” bugs.

## Consequences

- UI overlays blend correctly over engine viewports without “double sRGB” mistakes.
- The renderer’s behavior is predictable and debuggable across platforms and surface formats.
- HDR is not blocked: the registry reserves an encoding slot, but P0 remains focused on correctness.

## Future Work

- Add optional HDR surface support (platform-dependent) and define output transfer functions (PQ/HLG).
- Add explicit color space identifiers beyond sRGB (e.g. Display P3) if editor workflows require it.
- Add a “render target viewer” component in `fret-components` for inspecting engine textures with selectable transfer.
