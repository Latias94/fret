# ADR 1185: Environment Queries — Preference Extensions (v1)

Status: Proposed

## Context

ADR 1171 defines a mechanism for reading a **committed per-window environment snapshot** during
declarative rendering, with dependency tracking and cache-key participation.

For general-purpose applications (and editor-grade shells), we also need a few additional
environment-level signals that frequently affect layout, motion, and visual affordances:

- Text scaling / accessibility font size (e.g. “make text larger”).
- Reduced transparency preference (avoid frosted-glass / blur-heavy UIs).
- System accent color (best-effort; used for highlights and affordances).

Without a contract, ecosystems end up inventing incompatible ad-hoc seams (per-component globals,
hard-coded assumptions, or platform-specific hacks).

## Decision

### D1 — Extend the committed snapshot with optional preference keys

Extend the committed per-window environment snapshot (ADR 1171) with additional **optional**
fields. Runners provide best-effort values and may leave them unset/unknown when unavailable.

New keys:

- `text_scale_factor: Option<f32>`
  - A multiplier relative to a baseline of `1.0`.
  - Example: `1.25` means “~25% larger text”.
- `prefers_reduced_transparency: Option<bool>`
  - `Some(true)` means the user prefers reduced transparency effects.
- `accent_color: Option<Color>`
  - Best-effort system accent color.
  - This is not a theme token; it is an input signal for ecosystem policy.

### D2 — Mechanism remains in `crates/fret-ui`; policy in `ecosystem/fret-ui-kit`

Layering remains unchanged:

- `crates/fret-ui`: mechanism only (storage + committed values + dependency tracking + diagnostics).
- `ecosystem/fret-ui-kit`: policy helpers that translate optional signals into defaults.
- `ecosystem/*`: recipes call helpers and remain portable across runners.

### D3 — Diagnostics export

Diagnostics bundles SHOULD expose these fields under the stable schema path
`debug.environment` (best-effort), alongside existing ADR 1171 fields.

## Runner notes (best-effort)

- Web/wasm:
  - `prefers_reduced_transparency` MAY be derived from media queries when available.
  - `text_scale_factor` MAY be derived from computed root font size (best-effort).
  - `accent_color` is typically unavailable in a portable way and may remain `None`.
- Native:
  - Values may be `None` until reliable sources are wired for each OS.

## Consequences

- Ecosystem components can consistently respect reduced transparency and text scaling without
  leaking platform-specific logic into recipes.
- Future platform work can fill in best-effort values without changing the public mechanism seam.

## Non-goals

- This ADR does not define theme tokens or prescribe how themes map accent colors.
- This ADR does not guarantee that every platform can supply every key in v1.

