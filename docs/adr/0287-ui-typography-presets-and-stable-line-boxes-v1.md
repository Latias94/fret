# ADR 0287: UI Typography Presets and Stable Line Boxes (v1)

Status: Accepted

## Context

Fret’s text stack supports a configurable `TextStyle.line_height` and a `TextLineHeightPolicy`
(`ExpandToFit` vs `FixedFromStyle`). In practice we have two competing needs:

1. **UI control stability** (buttons, tabs, inputs, menus): single-line text should not change the
   control’s measured height when fallback fonts or emoji participate in shaping.
2. **Content correctness** (markdown/prose, rich text): text should avoid clipping and should
   accommodate taller glyphs when needed.

Historically, some UI surfaces observed “first line becomes taller” in otherwise fixed-height
controls. The typical root cause is that shaping picks a fallback run with larger ascent/descent,
and the line box expands when line height is derived from run metrics.

We must solve this without turning `crates/fret-ui` into a policy-heavy UI kit (ADR 0066).

## Goals

1. Provide a **single, reusable preset surface** for common UI typography (size + line height +
   stable line box policy).
2. Make “stable line box” the *default* for UI control text when `line_height` is explicitly set.
3. Keep policy and recipe decisions in the ecosystem layer (`fret-ui-kit`, `fret-ui-shadcn`).
4. Keep content surfaces free to choose a more permissive policy (`ExpandToFit`) when clipping
   would be unacceptable.
5. Add regression gates that lock the “no first-line jump” behavior for control text.

## Non-goals (v1)

- Replacing the entire text system (tracked separately in `docs/workstreams/standalone/text-system-v2-parley.md`).
- Defining full typographic features (hyphenation, justification, per-script fallback chains).
- Standardizing a single global “design system” for all ecosystems.

## Decision

### D1 — Treat stable line boxes as an ecosystem policy for control text

For “control text” (single-line labels used for layout in UI components), the ecosystem defaults
should prefer a fixed line box derived from style:

- `TextStyle.line_height: Some(...)`
- `TextStyle.line_height_policy: TextLineHeightPolicy::FixedFromStyle`

Rationale:

- This matches CSS/GPUI-like “half-leading” baseline placement expectations and avoids layout drift
  when a fallback run participates in shaping.
- It keeps the mechanism surface small: `TextLineHeightPolicy` remains a contract in `fret-core`,
  while the decision of when to use it lives in ecosystem presets/recipes.

### D2 — Introduce a preset vocabulary in `fret-ui-kit` (ecosystem)

`ecosystem/fret-ui-kit` should provide a stable preset surface for UI authors and component
ecosystems, centered around:

- size presets (xs/sm/base/prose),
- intended surface (control vs content),
- font family intent (ui vs monospace),
- and explicit guidance for vertical placement when bounds exceed line height (see D3).

This preset vocabulary is **not** added to `crates/fret-ui` (ADR 0066).

### D3 — Use “bounds-as-line-box” placement for vertically-centered fixed-height controls

For fixed-height controls whose allocated height is larger than the natural line box, authors
should opt into `TextVerticalPlacement::BoundsAsLineBox` so baseline placement follows a stable
half-leading model inside the allocated bounds.

The `fret-ui-kit` authoring surface should make this easy to apply for control text.

### D4 — Content surfaces may choose `ExpandToFit`

For prose/markdown/editor content where clipping is unacceptable, ecosystems may prefer
`TextLineHeightPolicy::ExpandToFit` even when a line height is configured. This allows the line box
to expand to accommodate taller fallback glyphs.

The choice of “control vs content” policy is explicitly an ecosystem decision.

## Relationship to upstream references (non-normative)

Flutter’s `StrutStyle` + `forceStrutHeight` is an example of a “paragraph-level stable line box”
mechanism. Fret’s `TextLineHeightPolicy::FixedFromStyle` serves a similar role for UI surfaces.

This ADR does not require Fret to mirror Flutter’s API shape; it only aligns on the underlying
intent: stable layout for UI controls, with opt-in correctness for content surfaces.

## Consequences

- Ecosystem components will converge on a shared typography vocabulary.
- UI control height becomes deterministic under emoji/fallback participation.
- Content surfaces retain an escape hatch to avoid clipping.

## Acceptance / gates

1. Add a targeted regression gate that renders a control label containing emoji + mixed scripts and
   asserts the control height and first-line line box remain stable.
2. Migrate `fret-ui-shadcn` control text constructors to use the `fret-ui-kit` preset surface where
   possible.
