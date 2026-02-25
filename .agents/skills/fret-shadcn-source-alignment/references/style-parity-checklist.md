# Style parity checklist (tokenized, deterministic; no new goldens by default)

Use this checklist when a shadcn-aligned component’s styling is “close enough” but still drifts in
subtle ways. The goal is to converge on stable, typed outcomes in a GPU-first renderer without
needing to expand golden coverage.

## 1) Token discipline (avoid magic numbers)

- Prefer typed tokens and vocab:
  - spacing: `Space`
  - radius: `Radius`
  - colors: theme keys / `ColorRef` (avoid hard-coded RGB unless it’s a fixture)
  - metrics: `MetricRef`
- If you must use `Px(...)`, ensure it’s one of:
  - a contract surface (e.g. hysteresis threshold, touch slop) with an ADR anchor, or
  - a single well-named constant shared across recipes.

Start points:

- Tokens/themes: `crates/fret-ui/src/theme.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- Style vocabulary: `ecosystem/fret-ui-kit/src/style/`

## 2) Border vs ring vs focus-visible (most common “looks off”)

Common drift causes:

- Ring implemented as a layout border (changes size) instead of paint-only.
- Focus ring clipped by `overflow: clip/hidden` on the wrong ancestor.
- Pixel snapping differences make 1px borders look blurry on some DPIs.

Checklist:

- Distinguish “layout border” from “paint-only ring”.
- Ensure focus-visible is modality-driven (keyboard vs pointer).
- Gate at least one deterministic invariant for focus ring visibility (not clipped).

Start points:

- Focus-visible contract: `docs/adr/0061-focus-rings-and-focus-visible.md`
- Pixel snapping: `crates/fret-ui/src/pixel_snap.rs`

## 3) Rounded clipping + overflow

DOM/CSS makes `overflow: hidden` + `border-radius` feel trivial. In a GPU-first renderer:

- clipping stacks are explicit,
- ordering affects shadows/rings,
- and incorrect clipping breaks overlays and outside-press behavior.

Checklist:

- Verify the clip applies to the intended subtree only.
- Confirm shadows and rings are rendered outside the clip when required by the design.
- If the component participates in overlay dismissal, ensure clipping does not hide hit-test regions unexpectedly.

Start points:

- Rounded clipping ADR: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- Rendering substrate: `docs/runtime-contract-matrix.md` (Rendering substrate section)

## 4) Shadows / elevation (quality vs perf trade)

Checklist:

- Decide if you need a real blur or a cheaper approximation (don’t accidentally regress perf).
- Keep shadow parameters tokenized (radii/offsets) instead of scattering per-component values.

Start points:

- Elevation ADR: `docs/adr/0060-shadows-and-elevation.md`

## 5) Typography + text layout

Subtle drift often comes from:

- line-height defaults,
- truncation/wrap behavior,
- baseline alignment in mixed-size rows,
- and font-metrics differences across platforms.

Checklist:

- Avoid forcing `nowrap` unless upstream does.
- Gate a layout invariant when text wrapping/truncation is user-visible.
- Ensure text selection/IME surfaces remain correct when changing paddings or transforms.

Start points:

- Text contracts: `docs/runtime-contract-matrix.md` (Text input engine section)

## 6) Interaction state styling (hover/active/disabled)

Checklist:

- Ensure hover styles do not activate for touch-first/can’t-hover pointers.
- Disabled state should affect both visuals and semantics (a11y flags).
- Pressed/active styles should not fight pointer capture/gesture arbitration.

Start points:

- Pointer capability helpers: `ecosystem/fret-ui-kit/src/declarative/pointer_queries.rs`
- A11y checklist: `docs/a11y-acceptance-checklist.md`

## 7) Responsive styling rules (when upstream is silent)

If upstream shadcn has no responsive rule, choose a stable driver:

- overlay “shell” decisions: prefer viewport/environment queries,
- panel content decisions: prefer container queries.

Then:

- add hysteresis around thresholds,
- define “unknown first frame” behavior (fallback style),
- gate resize behavior deterministically.

Start points:

- Container queries ADR: `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
- Environment/viewport ADR: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- Helpers: `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`,
  `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`

## 8) What to gate (preferred)

Prefer gates that remain valid across refactors:

- token-level invariants (e.g. padding/radius values applied via a stable recipe surface),
- geometry invariants (rect relationships, clamping, alignment),
- paint invariants only when the contract is stable (avoid brittle pixel-perfect checks).

Avoid:

- adding new web goldens as the default response to a styling mismatch.

