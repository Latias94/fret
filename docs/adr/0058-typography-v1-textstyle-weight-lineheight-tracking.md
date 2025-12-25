# ADR 0058: Typography v1 (TextStyle Weight, Line Height, Tracking)

Status: Accepted

## Context

Fret aims to support a Tailwind/shadcn-inspired component ecosystem (ADR 0037, ADR 0056). For that
to be practical, the framework must be able to express basic typography variants without baking
magic numbers into widgets:

- **font weight** (`font-medium`, `font-semibold`, ...)
- **line height** (`leading-*`)
- **tracking / letter spacing** (`tracking-*`)

Historically, `fret-core::TextStyle` only carried `font` and `size` (ADR 0006). That was enough to
shape and render text, but not enough to reliably reproduce shadcn-like visual language or to build
consistent component recipes.

## Decision

Extend the backend-agnostic text contract (ADR 0006) by expanding `fret-core::TextStyle`:

- Add `FontWeight(u16)` with common constants (`NORMAL`, `MEDIUM`, `BOLD`, ...).
- Add `TextStyle::line_height: Option<Px>` as an optional **logical-px** override.
- Add `TextStyle::letter_spacing_em: Option<f32>` as an optional tracking override in **EM**.

Renderer implications (ADR 0006 / ADR 0029):

- The text backend must include these fields in its cache key, so blobs for different typography
  parameters do not alias.
- Rasterization remains scale-factor aware via `TextConstraints.scale_factor`. Typography values
  are specified in logical units and scaled internally for shaping/rasterization.

## Consequences

- Components can express shadcn-style typography without inventing per-widget style structs.
- `FontWeight` is `fret-core`-native and does not require pulling `fontdb`/`cosmic-text` types into
  `fret-core`.
- Theme schema changes are **not required** for v1. Component recipes can set typography directly,
  and future work can add theme-level typography tokens/aliases (ADR 0050 follow-up).

## Notes / Future Work

- Theme-level typography tokens/aliases (e.g. global body/mono stacks, weight presets, line-height
  scales) remain planned work (see `docs/mvp/active-plan.md` "MVP 53").

