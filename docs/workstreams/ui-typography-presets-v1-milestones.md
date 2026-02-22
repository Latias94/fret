# Milestones: UI Typography Presets v1

This is **non-normative** and tracks delivery progress.

Status key:
- Done: exit criteria met for v1 scope.
- Partial: core landed, follow-ups tracked in TODO.

## M0 — Preset surface exists (API + docs)

Status: Done.

Exit criteria:

- `fret-ui-kit` exposes a stable preset vocabulary (control/content, ui/mono, xs/sm/base/prose).
- Preset docs explain when to use `BoundsAsLineBox`.
- At least one shadcn control uses the preset surface.

Evidence:
- `ecosystem/fret-ui-kit/src/typography.rs`
- `docs/workstreams/ui-typography-presets-v1.md`

## M1 — shadcn control text migrated (core set)

Status: Partial.

Exit criteria:

- `fret-ui-shadcn` core controls (button, menu item, input label, radio label) use presets/helpers.
- No remaining ad-hoc `TextStyle` literals in those components for control text sizing/line height.

Notes:
- Remaining “builder-chain” callsites that manually set `text_size_px` + `line_height_px` +
  `FixedFromStyle` should be audited and moved to intent/preset helpers (or at least add
  `BoundsAsLineBox` placement where the control uses a fixed height).
- Tracking: `docs/workstreams/ui-typography-presets-v1-todo.md`

## M2 — Regression gates in place

Status: Done.

Exit criteria:

- A targeted test/gate fails on “first-line jump” regressions.
- Gate is documented and linked from the workstream.

Evidence:
- `ecosystem/fret-ui-kit/tests/typography_real_shaping.rs`
- `ecosystem/fret-ui-material3/src/lib.rs`

## M3 — Intent-first API + material3 adoption

Status: Done.

Exit criteria:

- `fret-ui-kit` exposes an intent-first typography API (e.g. `TextIntent::Control/Content`) that
  returns a ready-to-use `TextStyle` (or builder) without per-component composition.
- `fret-ui-material3` control surfaces adopt the same stability defaults (fixed line boxes for
  controls; expand-to-fit for content).

Evidence:
- `ecosystem/fret-ui-kit/src/typography.rs` (`TextIntent`, `TypographyPreset`, `with_intent`)
- `ecosystem/fret-ui-material3/src/lib.rs` (token + shaping gates)

## M4 — Markdown + code-view adoption

Status: Partial.

Exit criteria:

- `fret-code-view` monospace line-aligned rows use control-intent helpers.
- `fret-markdown` defaults (content for prose; control for fixed chrome labels) are implemented and
  recorded in this workstream.

Notes:

- `fret-ui-editor` audit remains pending; track in `docs/workstreams/ui-typography-presets-v1-todo.md`.

Evidence:

- `ecosystem/fret-code-view/src/code_block.rs`
- `ecosystem/fret-markdown/src/lib.rs`
