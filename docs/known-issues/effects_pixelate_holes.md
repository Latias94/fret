# Pixelate Backdrop: Clear-Colored "Holes"

## Symptom
- In `effects_demo`, `EffectMode::Backdrop` + `EffectStep::Pixelate` can produce large clear-colored regions inside the effect bounds (typically an "L" shape).

## Root Cause
- The pixelation pipeline uses a downsample + upscale pair.
- Pixelation blocks are aligned to the framebuffer origin, but effect bounds/scissors may start/end mid-block.
- If we scissor the **downsample** pass, we can omit texels that are still sampled by the **upscale** pass, which shows up as clear-colored "holes".

## Fix
- Do not scissor the pixelate downsample pass. Keep the upscale pass scissored to the effect bounds.

## Debugging
- Enable RenderDoc capture integration:
  - `FRET_RENDERDOC=1`
  - `FRET_RENDERDOC_DLL=C:\path\to\renderdoc.dll` (on Windows, RenderDoc must be loaded in-process)
  - Optional: `FRET_RENDERDOC_AUTOCAPTURE=1` to capture the first rendered frame automatically
  - Captures are written under `.fret/renderdoc/`
- Force wgpu backend if needed: `FRET_WGPU_BACKEND=dx12|vulkan|metal|gl|all`

