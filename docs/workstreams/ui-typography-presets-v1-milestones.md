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

