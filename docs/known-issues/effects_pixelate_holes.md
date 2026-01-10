# Pixelate Backdrop: Clear-Colored "Holes"

## Symptom
- In `effects_demo`, `EffectMode::Backdrop` + `EffectStep::Pixelate` can produce large clear-colored regions inside the effect bounds (typically an "L" shape).

## Root Cause
- (Historical) The pixelation pipeline uses a downsample + upscale pair.
- When the sampling grid is effectively anchored to the framebuffer origin, effect bounds/scissors can start/end mid-block.
- If we scissor the **downsample** pass in that setup, we can omit texels that are still sampled by the **upscale** pass, which shows up as clear-colored "holes".

## Fix
- The current implementation avoids this class of artifact by anchoring pixelation to the effect bounds:
  - Downsample into an effect-local intermediate target.
  - Use origin-aware scaling params to map between effect-local and full-size coordinates.

## Debugging
- See:
  - `docs/renderdoc-inspection.md`
  - `docs/debugging-playbook.md`
