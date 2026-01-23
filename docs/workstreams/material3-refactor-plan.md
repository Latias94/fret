# Material 3 (and Expressive) Refactor Plan

Status: Draft / in-progress

This document is the **execution plan** for a “fearless refactor” of Material 3 alignment in Fret.
It complements `docs/workstreams/material3-todo.md`, which remains the broader tracking checklist.

## Motivation

We currently have multiple Material 3 components implemented inside `ecosystem/fret-ui-material3`,
but the *visual + interaction outcome* consistency is still uneven. The common failure mode is:

- Each component implements its own interaction orchestration (pressed/hover/focus, ripple, state
  layer) and token fallback behavior.
- When tokens are missing, components accidentally fall back to non-Material theme tokens (e.g.
  shadcn “foreground/card”), causing color mismatches.
- Some components create/remove indicator children conditionally, which can introduce subtle layout
  instability (visible as flicker).

In contrast, Compose Material3 achieves consistency by centralizing:

- interaction state in an `InteractionSource`-like substrate,
- ripple/state-layer in an `Indication`-like substrate,
- content color + typography via composition locals and controlled scope.

## Observed Divergences (so far)

These are outcome-level issues that typically indicate missing shared policy or missing core
mechanisms (measurement/rounding), not “a single wrong token”:

- **Radio indicator geometry**: selected dot size/position can drift at some scales if the geometry
  math is duplicated across components or lacks consistent pixel snapping.
- **Tabs press/click flicker**: commonly caused by conditional indicator insertion/removal or by
  indicator sizing that depends on last-frame bounds (needs a stable structure + stable
  measurement source).
- **Text/icon color mismatches**: often caused by missing scoped defaults (Compose-style
  `LocalContentColor`) and implicit fallbacks into non-Material token namespaces.
- **Ink bounds inconsistencies**: ripple/state-layer bounds should follow Material “touch target /
  state layer” rules and stay stable across states (pressed/hover/focus).

References in this repo:

- Compose Material3 `RadioButton`: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/RadioButton.kt`
- Compose Material3 tabs: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TabRow.kt`,
  `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tab.kt`
- Compose ripple/indication: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Ripple.kt`

## Target Architecture

All Material policies stay in **one crate**: `ecosystem/fret-ui-material3`.

### 1) Foundation layer (new)

Introduce a small “foundation” module set that components depend on:

- `foundation::interaction_source`
  - Canonical pressed/hover/focus state model.
  - Stable per-element runtime storage (keyed by pressable id).
  - Optional “hoist” surface for app code (for previews/tests).
- `foundation::indication`
  - A reusable “paint-only” indication that composes:
    - state layer (hover / focus / pressed) with token-driven alphas,
    - ripple (bounded/unbounded, pointer-origin rules).
  - Components should not manually run their own ripple/state-layer animators.
- `foundation::token_resolver`
  - Strict M3 token lookup:
    - `md.comp.*` → fallback to `md.sys.*` → last-resort fallback.
  - Explicitly avoid falling back into shadcn tokens during Material3 rendering (configurable).
- `foundation::content_color`
  - A scoped content color strategy (text/icon defaults) comparable to Compose’s
    `LocalContentColor`, without requiring a full composition-local system on day one.

### 2) Components become “thin recipes”

Each component should become:

- structure/layout + semantics + focus policy,
- calls into `foundation` for:
  - interaction state,
  - indication painting,
  - token resolution,
  - content defaults.

The goal is to reduce per-component bespoke logic and eliminate divergence.

## Scope and Breaking Changes

This refactor is allowed to be breaking within `ecosystem/fret-ui-material3` as needed.
We will prefer **mechanism preservation** in `crates/fret-ui` and keep core changes minimal, but
we will not hesitate to propose core changes if they are required for correct outcomes.

## Core Mechanism Gaps (candidates)

These are *candidates*, not guaranteed core work:

- **Structured corner sets**: implemented via `ThemeConfig.corners` + `Theme::corners_by_key` to
  represent Material corner-set tokens (e.g. `corner-extra-small-top`).
- **Scoped theme / content defaults**: a local theme overlay / “content color” propagation
  mechanism would reduce boilerplate and mismatch risk.
- **Subcompose-like measurement**: for tabs (indicator “match content size”) and other
  measurement-driven visuals, relying only on last-frame bounds can cause visible jitter.
- **Pixel snapping / rounding policy**: some controls require consistent rounding rules across
  layout + paint to avoid “drift” at non-1.0 scale factors.
- **Stable structure guarantees**: better guidance or helpers to keep indicator/ink layers present
  without conditional insertion/removal.

## Milestones

### M0 — Lock the invariants

- Define “no shadcn fallbacks” rule for Material3 components.
- Define strict token namespaces and the canonical fallback chain.
- Decide the public surface for hoisted interaction sources (if any).

### M1 — Introduce foundation modules

- Add `foundation` modules + internal APIs.
- Provide a small internal conformance harness (unit tests) for:
  - pressed/hover/focus state transitions,
  - ripple bounded/unbounded rules,
  - “no fallback to shadcn tokens” enforcement (where feasible).

### M2 — Migrate 2 components end-to-end

Pick two representative components:

- `Button` (baseline press/hover/focus + ripple/state-layer),
- `Tabs` (indicator + label colors + structural stability).

Goal: prove the foundation approach reduces divergence and removes flicker/mismatch classes.

### M3 — Migrate the rest

- `IconButton`, `Checkbox`, `Switch`, `Radio`, `TextField`, `Menu/MenuItem`, and new components.
- Remove duplicated per-component animators and ad-hoc fallbacks.

### M4 — Add alignment tracking and regression tooling

- Expand UI Gallery pages to cover “state matrix” views:
  - default/hover/focus/pressed/disabled/selected,
  - light/dark,
  - (later) Expressive variants.
- Add scripted interaction tests where feasible.
- Add token coverage tooling to detect drift:
  - `cargo run -p fret-ui-material3 --bin material3_token_audit -- --material-web-dir <path>`
  - This reports:
    - keys referenced by `fret-ui-material3` sources but missing from `tokens::v30` injection,
    - keys that do not exist in Material Web v30 sassvars (typos / wrong namespaces),
    - (optional) Material Web keys missing in our injection by component prefix.
- Add a token import generator to keep sys/comp tokens in sync with Material Web:
  - `cargo run -p fret-ui-material3 --bin material3_token_import -- --material-web-dir <path>`
  - This regenerates `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`.
  - Typescale injection is generated as composed `TextStyle` tokens and maps `md.ref.typeface`
    (`plain` vs `brand`) via `TypographyOptions`.
  - Shape injection is generated for `md.sys.shape.*`, including corner-set tokens via
    `ThemeConfig.corners` / `Theme::corners_by_key`.
  - Component scalar tokens are gradually generated by prefix (currently: `Button`, `Switch`,
    `IconButton`, `PrimaryNavigationTab`, `Menu`, `TextField`). Colors remain derived via `theme_config_with_colors`.

## Definition of Done (per component)

- Visual outcomes use **Material tokens** (`md.comp.*`, `md.sys.*`) and follow the strict fallback
  chain.
- Interaction outcomes:
  - state layer alpha matches `md.sys.state.*` roles,
  - ripple rules match bounded/unbounded expectations,
  - no layout flicker from structural changes.
- Semantics:
  - correct role, selected/checked, `pos_in_set`/`set_size` where relevant,
  - stable focus policy (roving behavior) for groups.
