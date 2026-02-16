# ADR 0056: Component Size/Density System (Tailwind-like Scales, GPUI-Inspired)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- gpui-component: https://github.com/longbridge/gpui-component
- shadcn/ui: https://github.com/shadcn-ui/ui
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted
Scope: Component ecosystem (`fret-components-*`), built on top of `fret-ui` + theme tokens (ADR 0032 / ADR 0050)

## Context

Fret already has a prototype **general-purpose UI kit** (`ecosystem/fret-ui-kit`) and a theme/token
foundation (ADR 0032 / ADR 0050). Dogfooding the UI kit exposed a recurring problem:

- visual density and control sizing drift between components (inputs/lists/buttons/tabs),
- UX tuning becomes “per-widget patching” (magic numbers), which does not scale,
- canvas/scene rendering makes DOM defaults (box-model clipping and line-height behavior) explicit; small
  mismatches are immediately visible (e.g. list highlight feeling “too tight”).

We want Fret’s component ecosystem to be:

- **general-purpose** (not editor-only),
- **tailwind/shadcn-friendly** in vocabulary (xs/sm/md/lg, spacing scales, variants),
- **GPUI-like** in ergonomics (one sizing system applied consistently).

gpui-component demonstrates an effective pattern:

- a `Size` enum (`xs/sm/md/lg`) as the shared vocabulary,
- helper methods like `list_px/list_py/input_h` to apply size consistently,
- a `Sizable` trait so components share a uniform `.with_size(...)` API.
- composable style refinements (`StyleRefinement`) and extension traits (`StyledExt`) so recipes/variants can be expressed as typed “style patches” (no utility string parser required).

Reference: `repo-ref/gpui-component/crates/ui/src/styled.rs` (search `Size`, `StyleSized`, `Sizable`).

## Decision

### 1) Introduce a shared `Size` for all components

Define a `Size` enum in the component ecosystem (recommended: `fret-ui-kit`), with the canonical set:

- `XSmall` (`xs`)
- `Small` (`sm`)
- `Medium` (`md`, default)
- `Large` (`lg`)

This is the **component-level** sizing vocabulary used by:

- buttons and icon buttons,
- text inputs/select/checkbox/switch/slider,
- list-like components (lists, command palette, trees, tables),
- toolbars/tabs/menu surfaces.

Components should not invent new sizing enums.

### 2) Define a single “control metrics” mapping per `Size`

Introduce a centralized mapping that derives common control metrics from `Size`, such as:

- input height and padding: `input_h`, `input_px`, `input_py`,
- list padding and row spacing: `list_px`, `list_py`, `row_gap_y`, highlight insets,
- icon size and gaps: `icon_size`, `icon_gap_x`,
- typography presets where relevant: `control_text_size`, `label_text_size`.

The mapping is a component ecosystem contract: it exists to keep spacing decisions consistent and easy to tune.

### 3) Tie sizing to theme tokens (typed baseline + namespaced overrides)

The sizing system consumes theme values via:

- typed baseline tokens (ADR 0050),
- optional namespaced dotted keys (ADR 0050 §5.1) for component ecosystems.

Rules:

- **Baseline**: use the typed theme tokens as a fallback (so the UI kit works without extra theme keys).
- **Typography baseline**: `control_text_size` falls back to the theme’s global base typography
  (`metric.font.size`, alias `font.size`), so components scale consistently when a theme changes font sizing.
- **Override**: allow overriding specific sizing-derived metrics via namespaced keys
  (e.g. `component.size.md.input_h`, `component.size.sm.list_py`) if we later need fine control.
- **List-specific metrics**: allow `metric.list.*` keys to remain as a stable escape hatch, but consume them
  through the sizing layer so list spacing stays consistent with other controls.

### 4) Provide GPUI-style ergonomics: `Sizable` + size helpers

Provide a uniform sizing API and helpers for component authors:

- `Sizable`: a trait so components share a consistent `.with_size(size)` style.
- `StyleSized` (or equivalent): helper methods that apply a `Size` to common styling “recipes”
  (e.g. `.list_px(size)`, `.input_h(size)`).

This does **not** imply a Tailwind class parser. The goal is a typed, Rust-native API that is Tailwind-like in
vocabulary and GPUI-like in usage.

### 5) Density is a theme-level extension, not a per-component knob

Global “density modes” (`compact/default/comfortable`) are a natural next step for large desktop apps, but
they are treated as a **theme-level** input (ADR 0032) rather than adding per-component flags everywhere.

This ADR locks the component-level `Size` contract first. Density is a follow-up design that can:

- scale the `Size -> control metrics` mapping, or
- select a different mapping table per density mode.

## Consequences

- UI kit work becomes scalable: spacing and sizing problems are fixed once in the sizing layer.
- Component APIs converge on a stable `Size` vocabulary compatible with shadcn/tailwind conventions.
- Theme authors can tune the “feel” of the UI without rewriting component code.
- Future component work (Command palette, Table, Tree, Combobox) inherits a consistent baseline.

## Alternatives Considered

### A) Per-component tokens only (no shared `Size`)

Pros:

- maximum per-component freedom.

Cons:

- quickly becomes unmaintainable (drift across controls),
- forces repeated “polish passes” as new widgets land,
- makes shadcn-like parity harder to achieve.

### B) Runtime Tailwind parser (CSS/utility strings)

Pros:

- familiar to frontend developers.

Cons:

- large surface area and runtime cost,
- hard to type-check and hard to keep deterministic in a retained scene renderer,
- not necessary to match Tailwind/shadcn *vocabulary*.

## Implementation Notes (Non-Normative)

- `Size` should be carried in component props and default to `Medium`.
- The mapping table can live in a small module (e.g. `fret-ui-kit::sizing`) and should be unit-tested.
- Avoid leaking sizing into `fret-core` or `fret-ui`; keep it in the component ecosystem.

## Current Status

- Prototype implemented in-tree:
  - `ecosystem/fret-ui-kit/src/sizing.rs` defines `Size` and `Sizable`.
  - Core UI kit components adopt `.with_size(...)` and derive paddings/heights from `Size`.
  - `fret-ui::VirtualList` exposes `set_style` / `set_row_height` so component wrappers can apply size-specific
    list styling without rebuilding the widget.

## References

- MVP tracking: `docs/archive/mvp/active-plan.md` (MVP 47)
- Theme/tokens: `docs/adr/0032-style-tokens-and-theme-resolution.md`, `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- gpui-component sizing ergonomics:
  - `repo-ref/gpui-component/crates/ui/src/styled.rs`
  - `repo-ref/gpui-component/crates/ui/src/list/list.rs` (search `.with_size(...)`)
- Zed/GPUI style extension traits (similar “typed style patches” ergonomics, non-normative): `repo-ref/zed/crates/gpui/src/styled.rs`
- shadcn/ui vocabulary reference:
  - `repo-ref/ui/apps/v4/content/docs/components/`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/`
