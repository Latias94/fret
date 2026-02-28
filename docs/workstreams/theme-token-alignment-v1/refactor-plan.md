# Theme Token Alignment v1 — Refactor Plan (Drift Prevention + Cross-Ecosystem Adoption)

This document turns the v1 taxonomy in `design.md` into an execution plan that scales to multiple
ecosystem crates (shadcn, AI elements, markdown/code-view, plot/charts, etc).

The goal is not “perfect Tailwind emulation”. The goal is:

- keep the contract surface small,
- keep authoring rules predictable,
- and make upstream parity bugs regress *less often* as new components land.

## Principles (what we optimize for)

1) **Stable keys, minimal palette**

- Semantic palette keys remain the primary contract (`background`, `foreground`, `muted`, `accent`,
  `primary`, `border`, `ring`, ...).
- Named literal colors remain minimal (`white`, `black`) and must be justified by upstream evidence.
- Component-derived tokens are allowed, but only when upstream rules differ by scheme/variant and
  cannot be represented as purely semantic or purely literal.

2) **Prefer “required tokens” at call sites**

When a recipe is *supposed* to work under a preset, it should generally read via
`Theme::color_token("...")` / `metric_token("...")` rather than `color_by_key("...")`.

Heuristic:

- If a key is part of the ecosystem “baseline contract”, treat it as required.
- If it is an app override hook (optional), use `*_by_key` and provide a well-defined fallback.

3) **Centralize alpha decisions**

Avoid per-component ad-hoc `alpha(accent, 0.5)` unless upstream is explicitly doing that. Prefer:

- preset-seeded derived tokens for `dark:*` deltas,
- or shared baseline keys such as `color.menu.item.hover`.

4) **Evidence-first gates**

- Prefer Rust tests for preset seeding outcomes (string keys -> resolved tokens exist).
- Use `fretboard diag` screenshot scripts for the “visual-risk” points (contrast, scrims, hover).
- Avoid grepping large diag artifacts (`bundle.json`); use `diag query/slice`.

## Shared interactive state tokens (recommended baseline set)

Across ecosystems, use these shared keys for common interaction states:

- `color.menu.item.hover`: hover background for menu-ish, chip-ish, action-ish rows.
- `color.list.row.hover`: hover background for list rows / tables / virtualized transcripts.
- `selection.background`: selection highlight (text selection / editor selection policy).
- `ring`: focus ring color.

If a recipe needs *different* behavior than these keys provide, prefer a `component.*` derived token
instead of introducing a new ad-hoc semantic key.

## Work phases

### P0 — Stop drift (mechanical guardrails)

- Adopt a “required token first” habit in ecosystem crates:
  - Prefer `theme.color_token("color.menu.item.hover")` over
    `theme.color_by_key("color.menu.item.hover").unwrap_or_else(|| theme.color_token("accent"))`.
- Ensure shadcn preset builders seed any “shared interactive keys” that must differ in dark schemes
  (e.g. `color.menu.item.hover` = `accent/50` in zinc/dark if upstream uses `dark:hover:bg-accent/50`).

Deliverables:

- One Rust test per seeded key (cheap, stable).
- A small diag screenshot for at least one consumer surface under zinc/dark.

### P1 — Close editor surfaces (Markdown + CodeView + Syntax)

These surfaces are reused by multiple ecosystems (AI, docs, editors).

- Markdown:
  - Inline code: match upstream `bg-muted` and gate it (zinc/dark).
  - Fenced code blocks: unify header/background/copy-button hover token usage.
- CodeView:
  - Copy button hover/pressed should use shared hover keys where applicable.
- Syntax:
  - Ensure token names map to stable theme keys; avoid embedding fixed colors in renderers.

Deliverables:

- At least one gate per surface class (inline code, code fence, copy button).

### P2 — Cross-ecosystem AI elements hardening

AI elements combine: chips, menus, list rows, code blocks, and “message bubbles”.

- Ensure transcript row highlights / hover backgrounds route through shared baseline keys.
- Prefer `fret.ai.*` keys only when upstream implies a stable “product hook” (e.g. user bubble bg).

Deliverables:

- A small set of zinc/dark screenshot gates for:
  - user bubble contrast,
  - sources block (`text-primary`),
  - action menu open state,
  - attachment chip hover state.

### P3 — Plot/charts and other heavy ecosystems

Data-viz components should not invent their own palette roles per component.

- Define a minimal component-derived namespace for plot/charts, e.g.:
  - `component.plot.axis.fg`, `component.plot.grid.fg`, `component.plot.series.1..n`, etc.
- Keep it layered:
  - shared semantics remain in `crates/fret-ui`,
  - plot policy remains in `ecosystem/*`.

Deliverables:

- A tiny “plot token contract” doc (what keys exist, who seeds them, what the fallbacks are).
- A small diag gate page in UI gallery to visually validate contrast.

### P4 — Optional automation (lint/drift checks)

Add guardrails only after we see repeated drift patterns.

Candidates:

- A lightweight check that flags new uses of:
  - `color_token("white")` / `color_token("black")` (stringly typed named colors),
  - `alpha(theme.color_token("accent"), ...)` in recipes that should use a shared key.
- Expand `tools/check_theme_token_coverage.py` to include a small denylist and/or report “new required keys”.

Current guardrail (small-by-default):

- `tools/check_theme_token_drift.py` fails if ecosystem/app code reads named literal colors via
  `theme.color_token("white")` / `theme.color_token("black")` (prefer `ThemeNamedColorKey` at call sites).

## Checklist for porting a new component (copy/paste)

1) Identify upstream intent:

- semantic vs literal vs `dark:*` derived.

2) Decide ownership:

- mechanism -> `crates/*`
- authoring glue -> `ecosystem/fret-ui-kit`
- recipes -> `ecosystem/*` component crate
- derived seeding -> preset builder (e.g. `shadcn_themes.rs`)

3) Pick tokens:

- semantic keys when intent is semantic,
- `ThemeNamedColorKey` only when upstream is literal,
- `component.*` only when upstream has a scheme/variant delta.

4) Add a gate:

- Rust test for seeding, or zinc/dark screenshot for contrast risk.

## Evidence anchors

- Taxonomy: `docs/workstreams/theme-token-alignment-v1/design.md`
- Inventory: `docs/workstreams/theme-token-alignment-v1/todo.md`
- Baseline theme keys: `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- Named colors policy: `docs/adr/0101-semantic-theme-keys-and-extensible-token-registry.md`
