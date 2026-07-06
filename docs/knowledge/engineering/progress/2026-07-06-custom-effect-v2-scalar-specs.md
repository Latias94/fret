---
type: "Work Progress"
title: "Custom Effect V2 scalar control specs"
description: "Work Progress for Custom Effect V2 scalar control specs."
timestamp: 2026-07-06T17:40:00Z
tags: ["fret", "custom-effect", "examples", "binding", "public-surface", "raw-model"]
git_branch: "refactor/custom-effect-scalar-spec"
verified_by: "cargo nextest run -p fret-examples custom_effect_v2_web_owner --no-fail-fast"
---

# Summary

Custom Effect V2 scalar controls now carry their numeric control spec:
`default`, `min`, `max`, and `step`.

# Details

- Added `CustomEffectV2ScalarSpec` to the shared Custom Effect V2 web helper.
- Changed `CustomEffectV2ScalarControl` so clamp fallback, rounded integer reads, reset defaults,
  and `Slider` range/step configuration all read from the same spec.
- Updated the web, identity, LUT, and glass-chrome demos to call `control.clamped_value(values)`
  and `control.slider()` instead of repeating range and step literals at every view/readout site.
- Aligned the web/LUT blur and corner-radius clamp ceilings with the app-facing slider ranges.
  Before this cleanup, the model read path accepted larger values than the UI could produce.
- Refreshed structure tests so regressions have to reintroduce either raw model exposure or
  call-site range/step drift.
- Added an owner-local render/semantics test proving `CustomEffectV2ScalarControl::slider()` applies
  the spec range and step to the actual shadcn slider semantics node.
- Kept source-policy classification production-focused by ignoring the owner helper's
  `#[cfg(test)]` raw seams instead of adding test-only `fret_ui`/`UiTree` seams to the helper record.

# Decision

This is the right next layer before a full custom-effect schema. A scalar spec is small enough to
remain a demo-local binding, but it removes the most fragile part of the current design: duplicated
numeric policy split across defaults, model reads, and UI controls. A full schema should still wait
until shader ABI slots, labels, docs, and diagnostics need to be shared across more surfaces.

# Verification

- `cargo fmt --all --check`
- `python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo check -p fret-examples --lib --tests`
- `cargo check -p fret-examples --target wasm32-unknown-unknown`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo nextest run -p fret-examples custom_effect_v2_web_owner --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

After this slice, the next custom-effect step should be a real schema design only if more demos need
shared shader ABI/control metadata.

# Citations

- [custom_effect_v2_web_owner.rs](../../../../apps/fret-examples/src/custom_effect_v2_web_owner.rs)
- [custom_effect_v2_web_demo.rs](../../../../apps/fret-examples/src/custom_effect_v2_web_demo.rs)
- [custom_effect_v2_lut_web_demo.rs](../../../../apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs)
- [custom_effect_overlay_text_surface.rs](../../../../apps/fret-examples/tests/custom_effect_overlay_text_surface.rs)
