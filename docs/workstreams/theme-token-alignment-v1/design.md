# Theme Token Alignment v1 (Semantic vs Named vs Component-Derived)

This workstream audits and migrates ecosystem components to a consistent theme-token rule set that
matches upstream shadcn/Tailwind intent while keeping Fret's contract boundaries intact.

The immediate motivator is upstream parity bugs where literal Tailwind classes (e.g. `text-white`,
`bg-black/50`, `bg-white`) were implemented via semantic palette keys, causing incorrect contrast
under specific presets (e.g. zinc/dark).

## Goals

- Define a clear decision rule for authors: when to use semantic tokens vs named literal colors vs
  component-derived tokens.
- Make upstream parity outcomes deterministic across presets (especially light/dark deltas like
  `dark:*`).
- Keep the compatibility surface minimal and strongly typed (avoid importing a full Tailwind
  palette into the theme contract).
- Add lightweight regression gates (tests and/or `fretboard diag` scripts) for high-signal visual
  outcomes.

## Non-goals

- Designing a full design-system palette for all apps (that belongs to app-level theming).
- Expanding named literal colors beyond what upstream parity demonstrably needs.
- Moving interaction policy into `crates/fret-ui` (mechanism-only stays mechanism-only).

## Token taxonomy (authoring rule set)

### 1) Semantic palette tokens (preferred)

Use semantic palette keys when the upstream intent is a semantic role:

- Examples: `background`, `foreground`, `muted-foreground`, `destructive`, `ring`, etc.
- Fret surface:
  - Typed: `ThemeColorKey::*`
  - Canonical string keys: shadcn semantic aliases (e.g. `"destructive"`, `"destructive-foreground"`)

Semantic tokens are about meaning, not about “literal white/black”.

### 2) Named literal colors (minimal compatibility surface)

Use named literal colors when upstream explicitly hardcodes a literal color (Tailwind class):

- `text-white` => `ThemeNamedColorKey::White`
- `bg-white` => `ThemeNamedColorKey::White`
- `bg-black/50` => `ThemeNamedColorKey::Black` + alpha adjustment

Rules:

- Keep the set small (start with `white`, `black` only).
- Literal colors must remain stable across presets unless a preset explicitly overrides them.
- Prefer a typed key (`ThemeNamedColorKey`) over ad-hoc string tokens in recipes.

### 3) Component-derived tokens (component-scoped overrides)

Use component-scoped tokens when upstream expresses a variant rule that is not purely semantic and
not purely literal, especially `dark:*` deltas or “recipe-specific” tweaks:

- Example (shadcn v4 Button destructive):
  - light: `bg-destructive`
  - dark: `dark:bg-destructive/60`
  - Encode this as `component.button.destructive.bg` seeded by the preset/theme loader.

Rules:

- Name format: `component.<component>.<variant>.<slot>` (keep it stable and discoverable).
- Seed in the ecosystem theme preset builder (e.g. shadcn preset generator), not in `crates/fret-ui`.
- Recipes may `color_by_key(..)` with a fallback to the semantic token (migration window).

## Layer ownership (non-negotiable)

- `crates/fret-ui`: theme mechanisms + stable typed keys and token resolution behavior.
- `ecosystem/fret-ui-kit`: authoring glue (`ColorRef`, `MetricRef`, token read surface).
- `ecosystem/*` recipe crates: policy/styling decisions and component-scoped token usage.
- Ecosystem theme preset builders (e.g. shadcn) own seeding `component.*` token defaults.

## Regression gates (definition of done)

For each migrated parity rule, leave at least one of:

- A targeted Rust test that asserts a resolved token outcome (preferred for token seeding rules).
- A `tools/diag-scripts/*.json` scenario capturing the visual risk point under zinc/dark.

Avoid “add a new golden by default”; prefer token/geometry invariants and diag screenshots for the
few places where literal colors matter.

## Refactor plan (how we keep drift from returning)

This workstream is not only a one-off parity sweep; it should leave behind a repeatable, low-drama
way to port new recipes without reintroducing “semantic vs literal” confusion.

See: `docs/workstreams/theme-token-alignment-v1/refactor-plan.md`.

## References

- ADR 0032 (tokens + resolution): `docs/adr/0032-style-tokens-and-theme-resolution.md`
- shadcn parity tracker: `docs/shadcn-declarative-progress.md`
- Tailwind semantics notes: `docs/tailwind-semantics-alignment.md`
- Repo refs: `docs/repo-ref.md` (upstream shadcn/ui v4 sources live under `repo-ref/ui`)
