# ADR 0228: Theme Value Kinds (Beyond Color + Px) and ThemeConfig v2

Status: Proposed

## Context

Fret currently models theme inputs as:

- colors: string → `Color` (linear)
- metrics: number → `Px`

This is sufficient for baseline editor theming and shadcn-style ports, but it is insufficient for
Material 3 / Expressive alignment, which requires additional token kinds:

- scalar numbers (e.g. state-layer opacity)
- durations (ms)
- easing curves (cubic-bezier)
- typescale / text styles (family, size, line-height, weight)

If these token kinds are not part of the theme system, component crates will reinvent ad-hoc
configuration maps, fragmenting style resolution and preventing consistent cross-surface theming.

## Decision

Extend the theme contract to support **typed token value kinds** beyond `Color` and `Px`.

### 1) Token kinds

Introduce additional token kinds:

- `Number` (unitless scalar `f32`)
- `DurationMs` (`u32` or `f32` milliseconds)
- `Easing` (cubic-bezier control points)
- `TextStyle` (a structured value: font family key, weight, size, line height, letter spacing)

### 2) Query API

Add query APIs on `Theme` / `ThemeSnapshot`:

- `number_by_key(&str) -> Option<f32>`
- `duration_by_key(&str) -> Option<Duration>`
- `easing_by_key(&str) -> Option<CubicBezier>`
- `text_style_by_key(&str) -> Option<TextStyle>`

### 3) ThemeConfig schema

Evolve `ThemeConfig` to be able to carry these kinds in a stable, file-authored form.

Options:

- **A: Add new top-level maps** (`numbers`, `durations_ms`, `easings`, `text_styles`).
- **B: Unified typed value map** (`values: HashMap<String, ThemeValue>`).

The v1 schema remains supported for a migration window.

### 4) Namespacing and compatibility

Continue to support stable string keys and namespacing conventions (ADR 0032 / ADR 0050).
Design-system-specific keys (e.g. Material tokens) must not pollute typed baseline enums; they are
queried by string key and resolved by the theme service.

## Consequences

- Enables a single coherent theme system for both shadcn and Material ecosystems.
- Avoids duplicated ad-hoc config surfaces for motion/state/typography.
- Keeps `crates/fret-ui` mechanism-only: the theme system is still a framework service, while
  design-system policy remains in ecosystem crates.
