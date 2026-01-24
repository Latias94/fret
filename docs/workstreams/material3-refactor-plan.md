# Material 3 (and Expressive) Refactor Plan

Status: In progress (foundation landed; migration ongoing)

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

Current state in this worktree:

- A `foundation` layer already exists (`indication`, `token_resolver`, `content`, `geometry`,
  `focus_ring`).
- Multiple components already use the foundation indication path (`Button`, `Tabs`, `Checkbox`,
  `IconButton`, `Switch`, `Menu`, `Radio`).
- Pressable indication timing defaults (durations + standard easing) are centralized in
  `foundation::indication::material_pressable_indication_config` to avoid per-component drift.
- A token import + audit pipeline exists to keep scalar tokens aligned with Material Web v30.

## Compose Multiplatform Baseline (Reference Architecture)

This section inventories the core "infrastructure" building blocks used by Compose Material3, so we
can make explicit boundary decisions in Fret (what belongs in `crates/fret-ui` vs
`ecosystem/fret-ui-material3` vs per-component recipes).

### Theming + tree-local overrides

Compose:

- `MaterialTheme` provides tree-local values via composition locals and supports partial overrides
  for subtrees.
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MaterialTheme.kt`
- `LocalContentColor` models “content defaults” (text/icon color) based on background surfaces.
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ContentColor.kt`
- A `MotionScheme` is also part of the theme and can be overridden per subtree.
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MotionScheme.kt`

Implication for Fret:

- We should keep `crates/fret-ui` mechanism-only, but we still need a **tree-local “Material
  context”** concept in `fret-ui-material3` so components can share consistent defaults without
  duplicating logic.
- The existing `ElementCx::inherited_state` mechanism is sufficient to implement subtree-scoped
  overrides in `fret-ui-material3` (provider pattern) without new core runtime concepts. Core work
  should only be considered if we hit hard limitations (e.g. ergonomics or missing invalidation
  hooks).

### Interaction → indication (state layer + ripple)

Compose:

- The default `LocalIndication` is a Material ripple, and components rely on a shared
  `InteractionSource` contract to coordinate pressed/hover/focus/dragged states.
- Ripple is implemented as an `IndicationNodeFactory` with:
  - bounded vs unbounded behavior,
  - optional fixed radius,
  - theme-aware defaults (uses `LocalContentColor` and state-layer alpha tokens),
  - a tree-local override (`LocalRippleConfiguration`) used as an escape hatch.
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Ripple.kt`

Implication for Fret:

- This maps directly onto our `foundation::indication` goal, but we should close the remaining
  parity gaps as *foundation work*, not per-component patches:
  - unbounded ripples,
  - keyboard activation ripple origin rules,
  - a scoped ripple configuration override (escape hatch).

### Motion scheme (spatial vs effects)

Compose:

- `MotionScheme` exposes 6 canonical specs: `{default, fast, slow} × {spatial, effects}`.
- The built-in schemes are implemented using springs with token-driven stiffness/damping
  (`StandardMotionTokens` / `ExpressiveMotionTokens`).
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MotionScheme.kt`

Implication for Fret:

- Our token model already supports durations/easings and scalar numbers, but we currently do not
  have a first-class “motion scheme” mapping. We should introduce a `foundation::motion_scheme`
  module that:
  - reads the relevant `md.sys.motion.*` / `md.comp.*.motion.spring.*` numbers,
  - converts them into reusable animator configs (spring or our closest equivalent),
  - centralizes “spatial vs effects” choices so components stop picking ad-hoc timings.
- If “spring” is required for parity, decide whether to implement it in `fret-ui-material3`
  (policy-heavy) or extend `crates/fret-ui` with a small, renderer-agnostic spring primitive.

Current progress:

- Material Web v30 `md.sys.motion.spring.*` tokens are imported into `ThemeConfig.numbers`.
- `fret-ui-material3` provides a small `SpringAnimator` in `crate::motion` and a `foundation::motion_scheme`
  token reader for the 6 canonical specs.
- `Button` and `IconButton` pressed shape now uses the spring path (corner radius morph), which acts as a
  conformance probe for MotionScheme plumbing.
- `Tabs` active indicator now animates via the same spring substrate (x/width/height) using a single
  container-level indicator. The indicator tracks the selected tab's last-known bounds (fallback:
  equal-split width), which mirrors the measurement-driven approach used by Compose `TabRow`.
- `Switch` thumb motion now uses the spring substrate (selected + pressed), replacing duration-based tweens.

### Tokens (typed access vs string keys)

Compose:

- Components typically read values from typed token objects (e.g. `ButtonTokens`, `CheckboxTokens`,
  `StateTokens`) rather than hardcoding raw keys.
- Many tokens resolve through `MaterialTheme.colorScheme.fromToken(...)` / `MaterialTheme.typography`
  / `MaterialTheme.shapes`, and produce the final per-state values.

Implication for Fret:

- We should keep “token resolution policy” in `fret-ui-material3` foundation (`token_resolver`,
  typed helpers), and keep raw string usage as a last resort.
- A practical Fret analogue is a set of typed helper modules (not necessarily 1:1 with Compose)
  that wrap:
  - key spelling,
  - fallback chain (`md.comp.*` → `md.sys.*`),
  - any derived tokens (e.g. disabled alpha multiplication).

## Boundary Decisions (What counts as “infrastructure”)

Rule of thumb:

- If multiple components must behave identically to avoid perceptual drift (ink, motion, focus,
  content defaults), it belongs in `ecosystem/fret-ui-material3` foundation.
- If the behavior requires engine guarantees (layout stability, hit test rules, clipping/shadows,
  theme scoping), it is a `crates/fret-ui` mechanism candidate.
- If the behavior is primarily structure/layout of one component, it stays in the component recipe.

**Mechanism candidates (`crates/fret-ui`)**

- Tree-local theme/content defaults overlay only if the provider pattern is insufficient for
  ergonomics or invalidation correctness.
- Precision-pointer hover semantics (ignore touch for hover tracking) to avoid "sticky" hover and
  interaction flicker in policy layers.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs`,
    tests in `crates/fret-ui/src/declarative/tests/interactions.rs` (`pressable_on_hover_change_hook_ignores_touch_pointer_move`),
    Compose reference: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/PrecisionPointer.kt`.
- Unbounded ripple clipping rules / paint support if current primitives are insufficient.
- Stable structural guidance/helpers if indicator insertion/removal causes flicker.

**Material foundation (`ecosystem/fret-ui-material3`)**

- `foundation::indication` (state layer + ripple) as the only supported ink orchestration path.
- `foundation::motion_scheme` (spatial/effects mapping + spring configs).
- `foundation::tokens` + `foundation::token_resolver` (typed access + strict fallback chain).
- `foundation::content` (content color defaults + disabled opacity conventions).
- `foundation::elevation` (MD3 level → shadow + tonal overlay mapping).
- `foundation::interactive_size` (minimum touch target enforcement + centered chrome).

**Component recipes (`ecosystem/fret-ui-material3/src/*.rs`)**

- Layout structure and measurement strategy.
- Accessibility semantics for the component surface (role/checked/selected, roving focus wiring).
- Per-state token selection *only when it is truly component-specific*.

### 1) Foundation layer (consolidate + extend)

The crate should have a small “foundation” module set that components depend on:

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
- `foundation::content`
  - Content default helpers today, and a scoped content color strategy (text/icon defaults)
    comparable to Compose’s `LocalContentColor` without requiring a full composition-local system on
    day one.

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
- **Scoped theme / content defaults**: implement via `ElementCx::inherited_state` provider patterns in
  `fret-ui-material3` first; consider core help only if invalidation/ergonomics require it.
- **Subcompose-like measurement**: for tabs (indicator “match content size”) and other
  measurement-driven visuals. We currently use a lightweight `foundation::layout_probe` that reads
  last-frame bounds (1-frame latency). If this proves insufficient (visible jitter), we should
  consider a core mechanism upgrade.
- **Pixel snapping / rounding policy**: some controls require consistent rounding rules across
  layout + paint to avoid “drift” at non-1.0 scale factors.
- **Stable structure guarantees**: better guidance or helpers to keep indicator/ink layers present
  without conditional insertion/removal.

## Milestones

### M0 — Lock the invariants

- [x] Define “no shadcn fallbacks” rule for Material3 components (regression test).
- [x] Define strict token namespaces and the canonical fallback chain (`md.comp.*` → `md.sys.*`).
- [ ] Decide the public surface for hoisted interaction sources (if any).

### M1 — Introduce foundation modules

- [x] Add `foundation` modules + internal APIs.
- [x] Provide a small internal conformance harness (unit tests) for:
  - pressed/hover/focus state transitions,
  - ripple bounded/unbounded rules,
  - overlay motion + modal focus outcomes (cubic-bezier transitions, focus trap/restore),
  - “no fallback to shadcn tokens” enforcement (where feasible).

### M2 — Migrate 2 components end-to-end

Pick two representative components:

- `Button` (baseline press/hover/focus + ripple/state-layer),
- `Tabs` (indicator + label colors + structural stability).

Goal: prove the foundation approach reduces divergence and removes flicker/mismatch classes.

### M3 — Migrate the rest

- `IconButton`, `Checkbox`, `Switch`, `Radio`, `TextField`, `Menu/MenuItem`, and new components.
- Remove duplicated per-component animators and ad-hoc fallbacks.

Status notes:

- Most migrated components now rely on `foundation::indication`.
- Dialog overlay motion is now implemented via shared opacity + render-transform wrappers (matching
  menu/tooltip patterns) to reduce drift.
- `TextField` still needs a full indication migration (it currently only avoids non-Material token
  fallbacks).

### M4 — Add alignment tracking and regression tooling

- [x] Expand UI Gallery pages to cover “state matrix” views:
  - default/hover/focus/pressed/disabled/selected,
  - light/dark,
  - (later) Expressive variants.
- [ ] Add scripted interaction tests where feasible.
- [x] Add token coverage tooling to detect drift:
  - `cargo run -p fret-ui-material3 --bin material3_token_audit -- --material-web-dir <path>`
  - This reports:
    - keys referenced by `fret-ui-material3` sources but missing from `tokens::v30` injection,
    - keys that do not exist in Material Web v30 sassvars (typos / wrong namespaces),
    - (optional) Material Web keys missing in our injection by component prefix.
- [x] Add a token import generator to keep sys/comp tokens in sync with Material Web:
  - `cargo run -p fret-ui-material3 --bin material3_token_import -- --material-web-dir <path>`
  - This regenerates `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`.
  - Typescale injection is generated as composed `TextStyle` tokens and maps `md.ref.typeface`
    (`plain` vs `brand`) via `TypographyOptions`.
  - Shape injection is generated for `md.sys.shape.*`, including corner-set tokens via
    `ThemeConfig.corners` / `Theme::corners_by_key`.
  - Component scalar tokens are gradually generated by prefix (currently: `Button`, `Checkbox`,
    `Switch`, `IconButton`, `PrimaryNavigationTab`, `Menu`, `TextField`). Colors remain derived via
    `theme_config_with_colors`.

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
