# Window Style Profiles (Ecosystem Recipes)

Window styles in Fret are intentionally split into:

- **Mechanism/contract facets** (`WindowStyleRequest`): orthogonal, capability-gated, diagnosable.
- **Profiles/recipes** (ecosystem-level): opinionated combinations of facets for common window
  postures (tool windows, HUD overlays, click-through overlays).

Profiles are **not** part of the portable contract surface. They live in ecosystem crates so apps
can iterate quickly without locking policies into `crates/`.

## Orthogonal facets (what to keep straight)

These are intentionally orthogonal. Avoid overloading "none" or "transparent" to mean multiple
things.

- Decorations (`WindowDecorationsRequest`)
  - `None` means **frameless / client-drawn decorations**.
  - It does *not* imply visual transparency, OS materials, or click-through.
- Background material (`WindowBackgroundMaterialRequest`)
  - `None` means **no OS-provided backdrop material**.
  - It does *not* imply frameless windows or visual transparency.
- Composited alpha surface (`transparent`)
  - `transparent=true` means **request a composited alpha window surface** (create-time; may be
    sticky).
  - It does *not* imply frameless windows or any specific backdrop material.
- OS hit-testing (`hit_test`)
  - `PassthroughAll` means **window-level click-through**.
  - `PassthroughRegions` means **click-through except for interactive regions** (ADR 0313).

## `fret-bootstrap` profiles (v1)

The bootstrap crate provides a small set of profiles as **recipes**:

- Feature gate: enable the `fret-bootstrap/window-style-profiles` Cargo feature.

- `fret_bootstrap::window_style_profiles::app_window_profile_v1`
- `fret_bootstrap::window_style_profiles::tool_window_profile_v1`
- `fret_bootstrap::window_style_profiles::hud_overlay_profile_v1`
- `fret_bootstrap::window_style_profiles::click_through_overlay_profile_v1`

Each profile returns a `CompiledWindowStyleProfileV1`:

- `style`: the requested `WindowStyleRequest` patch.
- `expectations`: best-effort expected *effective* facets after clamping (for diagnostics/scripts),
  including `hit_test_regions_fingerprint64` when regions are effective.

## Diagnostics gating (script-friendly)

Window style outcomes are observable via the runner window style diagnostics snapshot and can be
asserted in scripts using:

- `window_style_effective_is` with `style.hit_test` and (optionally) `style.hit_test_regions_fingerprint64`
- `window_background_material_effective_is`

For region-based hit testing, prefer asserting a stable fingerprint rather than raw geometry.

## Related ADRs

- Window styles: `docs/adr/0139-window-styles-and-utility-windows.md`
- Window background materials: `docs/adr/0310-window-background-materials-v1.md`
- Window hit testing (facet): `docs/adr/0312-window-input-hit-testing-and-passthrough-v1.md`
- Window hit-test regions: `docs/adr/0313-window-hit-test-regions-v1.md`
