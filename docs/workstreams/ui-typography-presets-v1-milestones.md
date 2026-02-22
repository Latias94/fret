# Milestones: UI Typography Presets v1

This is **non-normative** and tracks delivery progress.

## M0 — Preset surface exists (API + docs)

Exit criteria:

- `fret-ui-kit` exposes a stable preset vocabulary (control/content, ui/mono, xs/sm/base/prose).
- Preset docs explain when to use `BoundsAsLineBox`.
- At least one shadcn control uses the preset surface.

## M1 — shadcn control text migrated (core set)

Exit criteria:

- `fret-ui-shadcn` core controls (button, menu item, input label, radio label) use presets/helpers.
- No remaining ad-hoc `TextStyle` literals in those components for control text sizing/line height.

## M2 — Regression gates in place

Exit criteria:

- A targeted test/gate fails on “first-line jump” regressions.
- Gate is documented and linked from the workstream.

## M3 — Intent-first API + material3 adoption

Exit criteria:

- `fret-ui-kit` exposes an intent-first typography API (e.g. `TextIntent::Control/Content`) that
  returns a ready-to-use `TextStyle` (or builder) without per-component composition.
- `fret-ui-material3` control surfaces adopt the same stability defaults (fixed line boxes for
  controls; expand-to-fit for content).
