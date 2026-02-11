# ADR 0220: Ecosystem Component Style Override Surface v1

Status: Proposed

## Context

Fret’s framework layer (`fret-ui`) intentionally does **not** define a component library or design
system (ADR 0066). Ecosystem crates (e.g. `fret-ui-shadcn`, future `fret-ui-material3`) need a
public, consistent way to let downstream apps and kits:

- override interactive widget visuals,
- do so in a state-aware way (hover/active/focus-visible/disabled/selected/open),
- avoid copying full default style tables just to tweak a single state.

The v1 state resolution primitive (`WidgetStates` + `WidgetStateProperty<T>`) exists (ADR 0219),
but the **public component “*Style API” shape** is still a hard-to-change contract surface.

Flutter’s `ButtonStyle` / `WidgetStateProperty` pattern is a useful precedent: style fields are
optional, and per-state values can independently resolve to `null` to indicate “no override” so
the widget can fall back to its default style.

## Decision

Define a v1 public style override contract for ecosystem component libraries.

### 1) Style structs are per-component and live in ecosystem crates

Each interactive control exposes a `*Style` struct in its component crate (e.g. `ButtonStyle`,
`CheckboxStyle`). This keeps policy out of `fret-ui` while still providing a stable override
surface for user code and higher-level kits.

### 2) Stateful slots use “nullable per-state properties”

For any slot that varies by widget state (background/foreground/border/ring/etc.), use:

- `Option<WidgetStateProperty<Option<T>>>`
- Prefer using the alias `fret_ui_kit::OverrideSlot<T>` in Rust code for readability.

Meaning:

- outer `Option`: the entire slot is not overridden when `None`;
- inner `Option<T>`: a specific state may intentionally return `None` to indicate “no override for
  this state”, so the widget falls back to its default style for that state.

This enables partial overrides like “only change hover background” without re-specifying defaults.

### 3) Merge semantics are shallow and right-biased

Every `*Style` provides:

- `fn merged(self, other: Self) -> Self`

Rules:

- right-biased: when `other.<field>.is_some()`, it replaces `self.<field>`;
- no deep merge: a `WidgetStateProperty<...>` is treated as atomic.

The per-state “nullable” shape provides partial override ergonomics without requiring deep merge.

### 4) Default style remains policy-owned

Controls compute a default style from theme tokens/recipes/variants (policy), then apply overrides
at resolve-time:

- resolve override property → `Option<T>`
- if `Some`, use it
- else fall back to the default property’s resolved value

## Consequences

Pros:

- Stable and consistent user-facing `*Style` surfaces across ecosystem component libraries.
- Partial overrides are ergonomic and composable (override only the states you care about).
- No surprising deep-merge behavior; merges stay cheap and predictable.

Cons / risks:

- More `Option` plumbing in component code (resolve-time fallback logic).
- v1 is color/metric-centric; richer style types (e.g. typography, elevation, ink/ripple) may
  require follow-up contracts.

## Implementation Notes (v1)

Evidence anchors:

- State resolution primitive: `docs/adr/0219-state-driven-style-resolution-v1.md`
- Shared patterns: `docs/shadcn-style-override-patterns.md`
- Implementations: `ecosystem/fret-ui-shadcn/src/{button,checkbox,radio_group,select,slider,switch,toggle,toggle_group,tabs,input}.rs`
