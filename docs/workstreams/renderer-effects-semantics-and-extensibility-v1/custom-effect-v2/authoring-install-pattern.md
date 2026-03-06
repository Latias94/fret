---
title: Custom Effect V2 Authoring - Install + Registration Pattern
status: draft
date: 2026-02-28
scope: renderer, effects, extensibility, ecosystem-authoring
---

# Custom Effect V2 Authoring - Install + Registration Pattern

This note answers a practical ecosystem question:

> Can a component/effect author ship a library that consumers can install in “one line”, including
> registering the custom effect program and its input textures?

## Key constraints (contract reality)

- `EffectId` is **renderer-scoped and runtime-assigned**.
  - Do **not** hardcode numeric IDs and do not assume cross-run stability.
- Custom effect registration is **capability-gated**.
  - Some backends/adapters support CustomV1 but not CustomV2.
  - On WebGPU, a “valid WGSL module” is not enough: uniformity rules and filterable texture support matter.
- A CustomV2 program is still bounded:
  - exactly one `src_texture` (renderer-provided),
  - plus at most one **user image input** (`ImageId`).

## Recommended authoring shape

Ship an “effect pack” struct that:

- owns a `CustomEffectProgramV2` (lazy, cached registration),
- owns (or can lazily create) its `ImageId` inputs,
- exposes a small “installation” entrypoint and a stable runtime handle for your components.

### Minimal “one line install” (builder wrapper pattern)

In native desktop apps that use the integrated `fret` builder, you can genuinely make it one line by
wrapping the builder:

```rust
pub fn install_into<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::UiAppBuilder<S> {
    builder
        .install_app(install_app_globals)
        .install_custom_effects(register_custom_effects)
        .on_gpu_ready(upload_images)
}
```

Consumers then do:

```rust
let app = fret::App::new("my-app")
    .window("My App", (1100.0, 720.0))
    .view::<MyView>()?;
my_effect_pack::install_into(app).run()?;
```

This keeps the “install surface” one call while respecting when services exist (custom effects and
GPU resources are only available after the renderer is created).

### Web/WASM install shape (explicit GPU-ready step)

In `winit + wgpu + trunk` style harnesses, you typically have an explicit “GPU ready” callback
where you receive `&WgpuContext` and `&mut Renderer`. Treat this as the only place you are allowed
to:

- register custom effects (`register_custom_effect_v2`),
- upload textures and register images (`register_image`).

Your library can still expose a single entrypoint, but internally it will split into:

- `install_app(app: &mut App)` (globals/models/tokens)
- `gpu_ready_install(app: &mut App, context: &WgpuContext, renderer: &mut Renderer)` (effects + images)

## Registration and caching rules

Use `fret-ui-kit`'s helper to keep the pattern consistent:

- `fret_ui_kit::custom_effects::CustomEffectProgramV2`
  - caches the `EffectId`,
  - can be invalidated on renderer recreation (`invalidate()`),
  - reuses an existing `EffectId` if the exact WGSL was already registered (ref-counted registry).

Example shape:

```rust
#[derive(Debug)]
pub struct MyEffects {
    program: CustomEffectProgramV2,
    input_image: Option<ImageId>,
}

impl MyEffects {
    pub fn new() -> Self {
        Self {
            program: CustomEffectProgramV2::wgsl_utf8(include_str!("my_effect.wgsl")),
            input_image: None,
        }
    }

    pub fn gpu_ready_install(
        &mut self,
        effects: &mut dyn fret_core::CustomEffectService,
        context: &WgpuContext,
        renderer: &mut Renderer,
    ) {
        let _ = self.program.ensure_registered(effects);
        self.input_image = Some(upload_my_input_image(context, renderer));
    }
}
```

## WebGPU/wasm portability notes (author checklist)

- **Derivatives (`dpdx`/`dpdy`/`fwidth`) must be in uniform control flow** (Tint validation).
  - Avoid non-uniform early returns before derivative calls.
  - If you do SDF-based AA, structure your shader so derivatives are evaluated unconditionally and
    apply coverage via `select(...)` / `mix(...)` instead of branching.
- CustomV2 input image sampling assumes a filterable float sample type.
  - In the wgpu backend, CustomV2 registration is gated on `Rgba8Unorm` being filterable for the adapter.
- Pick the correct `ImageColorSpace` for your input:
  - `Linear` for data textures (noise/normal/LUT encoded in 2D),
  - `Srgb` for color images that should decode to linear during sampling.

## Evidence anchors in this repo

- Helper type: `ecosystem/fret-ui-kit/src/custom_effects.rs` (`CustomEffectProgramV2`)
- CustomV2 ABI + contracts: `docs/adr/0303-custom-effect-v2-user-image-input.md`
- CustomV2 implementation tracker: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/README.md`
- A Web/WASM “starter” demo that follows this pattern:
  - `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs`
- A native desktop demo that shows a true “one call install” wrapper:
  - `apps/fret-examples/src/custom_effect_v2_demo.rs` (`install_into`)
