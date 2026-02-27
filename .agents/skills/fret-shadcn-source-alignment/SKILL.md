---
name: fret-shadcn-source-alignment
description: "This skill should be used when the user asks to \"align shadcn components\", \"match Radix behaviors\", \"debug parity mismatches\", or \"port upstream shadcn/ui v4 recipes\". Provides an upstream-alignment workflow (shadcn/Radix/Base UI) that maps changes to the correct Fret layer and locks outcomes with targeted tests and `fretboard diag` scripts."
---

# Shadcn / Radix source alignment

## When to use

- A shadcn/Radix-inspired component “doesn’t behave like upstream” (dismiss/focus/keyboard nav/placement).
- You need to decide whether a fix belongs in `crates/fret-ui` vs `ecosystem/fret-ui-kit` vs `ecosystem/fret-ui-shadcn`.
- You fixed a mismatch once and want to lock it with tests and/or `fretboard diag` scripted repros.

## Choose this vs adjacent skills

- Use this skill when the goal is **upstream parity** (Radix semantics / shadcn composition) plus a regression gate.
- Use `fret-app-ui-builder` when you just need a good recipe for building UI (not necessarily parity work).
- Use `fret-diag-workflow` when the main deliverable is a repro/gate for a bug (and parity is secondary).
- Use `fret-ui-review` when the request is an audit of app UI code quality and layering (not a specific parity mismatch).

## Inputs to collect (ask the user)

Ask these to keep the work scoped and landable:

- Which component + mismatch class (dismiss/focus/keyboard nav/placement/style)?
- Which mechanism axis is likely involved (overlay dismissal/focus restore/hit-testing/transform/clipping/breakpoints)?
- What is the upstream source of truth (Radix docs vs shadcn composition/source)?
- Which layer should own the change (mechanism vs policy vs recipe)?
- What regression protection is required: unit test, parity harness case, and/or diag script?
- Do we need a new stable `test_id` surface for automation?
- What platforms and input types must match (native/web; mouse/touch/pen)?
- Does parity include accessibility outcomes (roles/relations/active-descendant/collection metadata)?
- Does the component rely on responsive breakpoints (Tailwind-like viewport or container queries)?
- Is this an overlay family that needs `disableOutsidePointerEvents`, safe-hover corridors, or touch slop rules?

Defaults if unclear:

- Treat interaction semantics as Radix truth; treat composition/sizing/tokens as shadcn truth; add at least one gate.
- When DOM-focused assumptions are involved, consult Base UI as an additional headless reference for part composition and accessibility patterns.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin components_gallery`
- Alternate (shadcn gallery + a11y harness surface): `cargo run -p fret-demo`

## Quick start

1. Identify the layer (mechanism vs policy vs recipe) before touching code.
2. Compare against upstream docs/source (shadcn for composition + sizing; Radix for semantics).
3. Land a gate: a small invariant test and/or a `tools/diag-scripts/*.json` scripted repro with stable `test_id`.

## Workflow

### 0) Run the mechanism checklist first (don’t chase pixels yet)

When shadcn “looks almost right”, the remaining drift is usually **mechanism** (overlay routing,
dismissal/focus, hit-testing, breakpoints), not styling. Before adding/adjusting web goldens, run:

- `.agents/skills/fret-shadcn-source-alignment/references/mechanism-parity-checklist.md`
- `.agents/skills/fret-shadcn-source-alignment/references/style-parity-checklist.md`

### 1) Map the mismatch to the right layer

- `crates/fret-ui`: mechanisms/contracts (tree/layout/semantics/focus/overlay roots).
- `ecosystem/fret-ui-kit`: headless policy + reusable infra (roving focus, typeahead, overlay policy).
- `ecosystem/fret-ui-shadcn`: shadcn v4 taxonomy + recipes (composition + styling).

If the mismatch is “interaction policy” (dismiss rules, focus restore, hover intent, menu navigation),
it almost never belongs in `crates/fret-ui`.

### 1.25) Pick the upstream “reference stack” explicitly

Use the right source for the right kind of parity:

- **APG**: keyboard navigation and composite widget semantics (tabs, menu, listbox/combobox, etc.).
- **Radix**: overlays (dismiss/focus/portal/presence) and nuanced interaction outcomes.
- **Floating UI**: placement/flip/shift/arrow geometry outcomes.
- **cmdk**: command palette interaction details.

See `docs/reference-stack-ui-behavior.md` for the repo’s priority order and boundary mapping.

### 1.5) Motion parity in a custom renderer (important)

shadcn/Radix sources are DOM-first and often assume CSS/Framer Motion behavior. Fret is a GPU-first
custom renderer, so you **cannot** port the runtime model 1:1. The goal is parity in *outcomes*:
timing, sequencing, interruption rules, and accessibility behavior.

Rules of thumb:

- Treat upstream motion (Framer Motion variants, CSS transitions) as a **UX spec**, not an API.
- Prefer ecosystem motion drivers (`ecosystem/fret-ui-kit`) over ad-hoc per-component math.
- Keep motion tunable via theme tokens (durations/easings/springs), not hard-coded numbers.
- Be explicit about hit-testing semantics:
  - DOM `transform` moves hit-testing with visuals; in Fret, choose `RenderTransform` when you want
    hit-testing to track motion, and `VisualTransform` when you want paint-only motion.
- Lock motion-sensitive changes with a deterministic diag gate (`--fixed-frame-delta-ms 16`).

### 1.6) Renderer parity (CSS/Tailwind → GPU-first self-rendering)

shadcn/Radix sources assume DOM + CSS. In Fret, “matching upstream” often means translating CSS
idioms into explicit scene operations and layout/paint contracts.

**Tailwind layout constraints parity (don’t chase pixels yet)**

shadcn UI “looks wrong” surprisingly often because a few layout constraints went missing during the
port (classic symptoms: shrink-to-content containers, narrow columns, aggressive word wrapping).

Before doing token work, verify the Tailwind → Fret constraint mapping is applied consistently:

- `w-full` / `h-full`
- `flex-1` (i.e. `flex: 1 1 0%`) + `min-w-0`
- `items-stretch` vs `items-center` defaults
- `overflow-x-auto` (often modeled as `ScrollArea(axis=X, type=Auto)`)

Cheatsheet (canonical, kept in the app-builder skill for reuse):

- `.agents/skills/fret-app-ui-builder/references/mind-models/mm-layout-and-sizing.md`

Use this mini playbook to decide where to implement a visual parity fix and when it’s worth adding
a new render primitive.

**Rule of thumb**

- If it’s purely a recipe/style choice, keep it in `ecosystem/fret-ui-shadcn` (tokens + chrome).
- If it’s a reusable style vocabulary or shaping behavior, put it in `ecosystem/fret-ui-kit`.
- If it requires a new draw primitive or cross-backend correctness, it likely belongs in the
  `fret-core` scene contract + renderer(s), then expose it via `crates/fret-ui` authoring surfaces.

**Common CSS → Fret translations**

- `border-radius` / rounded corners
  - Prefer first-class rounded-rect ops over “masking by many quads” when quality matters.
  - Start points: `crates/fret-core/src/scene/`, `crates/fret-render-wgpu/` encode path.
- `border` / `ring` / outline
  - Distinguish “layout-affecting border” vs “paint-only ring” and lock the focus-visible contract.
  - Evidence: `docs/adr/0061-focus-rings-and-focus-visible.md`, `crates/fret-ui/src/pixel_snap.rs`.
- `box-shadow` / elevation
  - Decide whether you need a real blur (expensive) vs a cheap drop shadow approximation.
  - Prefer stable tokenized radii/offsets; gate with a screenshot or a small pixel/geometry
    invariant when feasible.
- `transform` animations
  - Choose between `RenderTransform` (moves hit-test) and `VisualTransform` (paint-only), then gate
    pointer outcomes (hover/outside-press) as well as visuals.
- `overflow: hidden` + rounded clipping
  - Explicit clip stacks are easy to get subtly wrong; gate scroll/overlay interactions where
    clipping affects dismissal or pointer occlusion.

**When to add a new render primitive**

Add/extend a scene op only if at least one is true:

- You can’t reach the upstream outcome with existing ops without major quality/perf regression.
- Multiple shadcn/Radix components need the same capability (e.g. dashed borders, crisp rrect
  stroke, blur variants).
- The behavior must be identical across backends (wgpu/WebGPU), so “just do it in a component”
  would fork correctness.

If you do add a primitive, require a gate at the same layer:

- Contract-level unit test (scene op encoding / invariants), and
- At least one consumer-level usage anchor (a shadcn recipe + a diag script or parity case).

### 1.65) Semantic conflict hazards (avoid “fighting” sources of truth)

As Fret gains more query surfaces (viewport snapshots, container regions, theme metadata, slot
scoping), the most expensive regressions come from *semantic conflicts*: the same decision is
implicitly driven by multiple “truths” that do not update at the same cadence.

Use these rules to keep parity work stable and refactors landable.

**Single source of truth per decision**

- For each responsive decision point, choose exactly one driver:
  - **viewport** (device shell / Tailwind `sm|md|lg` semantics), or
  - **container region** (panel width / docking resize).
- If both are needed, expose an explicit recipe-level knob (defaulting to web parity) and gate both
  modes. Do not silently “mix and match” in one layout subtree.

**Overlays and circular sizing**

- In overlay slots (popover/menu/dialog/sheet), prefer viewport queries for breakpoint-like
  decisions unless you can prove the container region is stable.
- Watch for “content decides overlay size, overlay size decides content breakpoint” loops. If you
  see resize jitter or non-deterministic first frame layout, treat it as a semantic conflict and
  add a targeted gate.

**Theme vs environment**

- Prefer app-owned theme metadata (e.g. `Theme.color_scheme`) for “dark vs light” styling decisions.
- Treat per-window environment hints (OS color scheme) as inputs to theme selection, not as a
  long-lived styling oracle inside recipes.

**Transforms and hit-testing**

- If a visual change affects interaction (hover/outside-press/drag handles), use
  `RenderTransform`. If it is paint-only, use `VisualTransform` and gate pointer outcomes
  explicitly.

**Stability mechanisms**

- Use hysteresis for breakpoint thresholds (viewport/container) to prevent flicker during resize.
- Prefer snapshot-style reads for “hot path” token/query reads and keep invalidation scopes tight.

**Regression protection**

- Always add at least one gate for a semantic decision:
  - invariant test for layout/semantics, and/or
  - `tools/diag-scripts/*.json` that exercises resize/dismiss/focus outcomes via stable `test_id`s.

### 1.75) Pointer / hit-testing / drag / cursor parity (GPU-first gotchas)

Common DOM-to-Fret translation points to check:

- **Hit-testing follows layout bounds**; transforms and clipping are explicit. Start at:
  - `crates/fret-ui/src/tree/hit_test.rs`
  - `crates/fret-ui/src/element.rs` (`VisualTransform` vs `RenderTransform`)
- **Pointer capture and observer passes** matter for overlays and drags:
  - Outside-press observer pass + click-through/consume behavior: `crates/fret-ui/src/tree/ui_tree_outside_press.rs` (ADR 0069).
  - Window-level dispatch (pointer occlusion / hover suppression): `crates/fret-ui/src/tree/dispatch/window.rs`.
- **Touch is not mouse**: outside-press is delayed to pointer-up with a slop threshold:
  - `crates/fret-ui/src/tree/ui_tree_focus.rs` (`TOUCH_POINTER_DOWN_OUTSIDE_SLOP_PX`).
- **Cursor icons are a contract surface** (especially for resize/drag handles):
  - `crates/fret-core/src/cursor.rs`
  - `crates/fret-runner-winit/src/mapping/cursor.rs`

If the mismatch is “dragging feels wrong”, prefer expressing policy via action hooks rather than
adding runtime toggles (see `docs/action-hooks.md` and `crates/fret-ui/src/action.rs`).

### 1.9) A11y parity (semantics snapshot outcomes, not DOM attributes)

Fret models accessibility via a semantics snapshot (portable schema) bridged by platform backends
e.g. AccessKit. For shadcn/Radix-aligned work, high-signal invariants include:

- roles (`SemanticsRole`) and flags (disabled/selected/expanded/checked),
- relations (`labelled_by`, `described_by`, `controls`),
- composite widgets (`active_descendant`),
- collections (`pos_in_set` / `set_size`) for menus/listbox-like surfaces.

Start points:

- Schema: `crates/fret-core/src/semantics.rs`
- Trigger stamping helpers (expanded/controls/described-by): `ecosystem/fret-ui-kit/src/primitives/trigger_a11y.rs`
- Manual acceptance (overlays + shadcn demo): `docs/a11y-acceptance-checklist.md`

### 1.95) Responsive / breakpoint parity (Tailwind-like, but not CSS)

When upstream uses responsive classes or container queries, prefer the in-tree helpers:

- Container queries (Tailwind-compatible breakpoints + hysteresis): `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`
- Viewport/device queries (Tailwind-compatible breakpoints + hysteresis): `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`
- shadcn recipes that depend on breakpoints (examples live under `ecosystem/fret-ui-shadcn/src/`).

### 1.97) Visual / token parity (Tailwind → typed theme tokens)

For shadcn “looks right” work, prefer token- and vocab-level alignment over per-component literals:

- Theme ingestion/conversion (shadcn v4 presets): `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- Style vocabulary (Space/Radius/ColorRef/MetricRef): `ecosystem/fret-ui-kit/src/style/`
- Focus-visible + rings (web-like outcomes, keyboard modality sensitive): `docs/adr/0061-focus-rings-and-focus-visible.md`
- Pixel snapping (crisp 1px borders/lines): `crates/fret-ui/src/pixel_snap.rs`

### 2) Find the upstream reference (source of truth)

1. Start with public docs (good enough for most alignment work):
   - shadcn components: https://ui.shadcn.com/docs/components
   - Radix primitives: https://www.radix-ui.com/primitives/docs/components
   - Base UI (headless primitives): https://base-ui.com/react/overview/quick-start
2. If you need exact implementation details, use source code:
   - shadcn/ui v4 source (New York v4 registry): https://github.com/shadcn-ui/ui/tree/main/apps/v4/registry/new-york-v4/ui
   - Radix Primitives source: https://github.com/radix-ui/primitives/tree/main/packages/react
   - Base UI source: https://github.com/mui/base-ui/tree/main/packages/react/src

If you maintain local upstream checkouts (or snapshots) for faster navigation, treat them as optional
convenience only. The skills bundle must remain usable from a clean checkout or a consumer app repo.

Use shadcn to learn the *composition + Tailwind contracts* (sizes, spacing, tokens), and Radix to
learn the *interaction semantics* (focus, dismiss, keyboard nav, portal layering).

Use Base UI as an additional reference for:

- part-based component composition (Root/Trigger/Content patterns),
- accessibility-first defaults (labels/fields/form patterns), and
- headless state machines that must be translated to Fret’s custom renderer (semantics/hit-testing/focus routing).

### 3) Align + protect with tests (goldens are not enough)

1. Prefer fast, targeted Rust unit tests for invariants.
   - Add tests next to the component in `ecosystem/fret-ui-shadcn/src/<component>.rs` (`#[cfg(test)]`).
   - Assert relationships (centering, overlap, clamping, tab order, active descendant, etc.).
2. Style/layout mismatches: prefer **Fret-side** deterministic checks (tokens, geometry invariants, and
   state-driven assertions) over adding new web goldens.
   - If a surface is already covered by an existing web-vs-fret harness, keep it green, but don’t
     default to expanding golden coverage during mechanism work.
3. Add an interaction repro script when state machines are involved.
   - Create `tools/diag-scripts/<scenario>.json` and gate it via `fretboard diag run`.
   - Always add/keep stable `test_id` targets in the Fret UI so scripts survive refactors.
4. Add an accessibility gate when semantics are involved.
   - Prefer invariant-style tests (roles/flags/relations/collection metadata).
   - Keep AccessKit mapping coverage green: `cargo nextest run -p fret-a11y-accesskit`.

Motion token guidance (ecosystem-level; keep stable for parity work):

- shadcn durations/easing (existing numeric scale):
  - `duration.shadcn.motion.{100|200|300|500}`
  - `easing.shadcn.motion`
- shadcn semantic keys (preferred for long-term authoring; numeric fallback):
  - `duration.shadcn.motion.overlay.{open|close}`
  - `easing.shadcn.motion.overlay`
  - `duration.shadcn.motion.sidebar.toggle`
  - `easing.shadcn.motion.sidebar`
- shadcn spring tokens (duration+bounce, Flutter-style):
  - `duration.shadcn.motion.spring.drawer.settle` + `number.shadcn.motion.spring.drawer.settle.bounce`
  - `duration.shadcn.motion.spring.drawer.inertia_bounce` + `number.shadcn.motion.spring.drawer.inertia_bounce.bounce`
- Material 3 scheme tokens (damping+stiffness):
  - `md.sys.motion.spring.{default|fast|slow}.{spatial|effects}.{damping|stiffness}`

### 4) High-value regression targets (start here)

- Overlay families: `dropdown-menu`, `select`, `context-menu`, `tooltip`/`hover-card`, `dialog`/`sheet`, `navigation-menu`.
- Listbox-ish behavior: roving focus, typeahead, active-descendant semantics, scroll clamping in constrained viewports.
- Responsive decisions: viewport vs container driver, hysteresis around thresholds, and “constrained viewport” max-height/scroll outcomes.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (smallest surface), Gate (script/test/parity), Evidence (upstream refs + anchors). See `fret-skills-playbook`.
- A clear layer mapping in the change (no “policy knobs” added to `crates/fret-ui` unless it is truly a mechanism).
- At least one regression artifact:
  - **state machine** mismatch ⇒ `tools/diag-scripts/*.json` repro with stable `test_id`,
  - **layout/style** mismatch ⇒ deterministic invariant test (tokens/geometry/paint outcomes).
- Evidence anchors in the PR/commit message: upstream link(s) + in-tree file(s) + test/script path(s).

## Evidence anchors

- Layers and contracts: `docs/architecture.md`, `docs/runtime-contract-matrix.md`
- Reference stack (APG/Radix/Floating/cmdk): `docs/reference-stack-ui-behavior.md`
- Shadcn parity tracker (canonical; treat older audits as historical): `docs/shadcn-declarative-progress.md`
- Mechanism checklist (this skill): `.agents/skills/fret-shadcn-source-alignment/references/mechanism-parity-checklist.md`
- Style checklist (this skill): `.agents/skills/fret-shadcn-source-alignment/references/style-parity-checklist.md`
- Action hooks (component-owned policy): `docs/action-hooks.md`
- Overlay ADRs:
  - `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
  - `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
  - `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- Queries:
  - Container queries (frame-lagged layout queries): `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
  - Environment/viewport snapshots: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- A11y acceptance checklist: `docs/a11y-acceptance-checklist.md`
- Local shadcn component implementations: `ecosystem/fret-ui-shadcn/src/`
- Policy primitives (roving/typeahead/overlays): `ecosystem/fret-ui-kit/src/primitives/`
- Responsive helpers:
  - `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`
  - `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`
- Existing web-vs-fret harness (optional, for already-covered surfaces): `ecosystem/fret-ui-shadcn/tests/`

## Examples

- Example: align a component with upstream shadcn/Radix behavior
  - User says: "Our Select/Popover differs from Radix—match the behavior."
  - Actions: choose upstream source-of-truth, implement in the correct Fret layer, and lock with scripts/tests.
  - Result: parity improvement with a regression gate.

## Common pitfalls

- Fixing policy mismatches by adding runtime knobs in `crates/fret-ui` (wrong layer most of the time).
- Relying on goldens alone for state-machine behavior (add a scripted repro).
- Missing stable `test_id` targets, causing scripts to rot during refactors.
- Mixing “parity work” and “new design work” without leaving any regression protection behind.
- Treating Base UI as a 1:1 “implementation port”: use it as a headless reference, then translate to Fret’s GPU-first renderer (semantics/hit-testing/focus routing).

## Troubleshooting

- Symptom: upstream behavior is subtle (focus/keyboard/ARIA).
  - Fix: gate semantics and interaction flows before chasing pixels.
- Symptom: a “visual” mismatch keeps reappearing.
  - Fix: make it a token- or invariant-level gate (don’t rely on ad-hoc tweaks).

## Related skills

- `fret-app-ui-builder` (recipes + stable `test_id` conventions)
- `fret-diag-workflow` (bundles + scripted repro gates)
- `fret-ui-review` (audit lens)
