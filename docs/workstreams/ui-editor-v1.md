# Editor Component Surface (`fret-ui-editor`) v1

Status: Proposed (workstream note; not an ADR)  
Last updated: 2026-02-15

## Summary

This workstream defines a new ecosystem crate, `ecosystem/fret-ui-editor`, that provides **editor-grade UI controls**
for inspector/workspace-style applications (Unity/Unreal/Godot-like). The goals are:

- A stable, reusable set of “precision” editor controls (numeric scrubbing, property grid, color edit, etc.).
- A clear separation of **mechanisms vs policies** (keep runtime contracts in `crates/fret-ui`; keep editor policies in ecosystem).
- A composable surface that can be “skinned” by multiple design systems (e.g. shadcn, material) without forcing a fork.

Primary constraints / references:

- Runtime contract boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Unified authoring surface: `docs/adr/0160-unified-authoring-builder-surface-v1.md`
- imui is an authoring frontend (not a second runtime): `docs/workstreams/imui-authoring-facade-v2.md`

## Scope

In scope (initial):

- Numeric editing primitives with editor-grade “hand feel”:
  - drag-to-change (scrub), modifier-based precision/acceleration, double-click to type, commit/cancel
  - undo/redo coalescing support (optional integration with `ecosystem/fret-undo`)
- Inspector building blocks:
  - `PropertyRow`, `PropertyGroup`, `PropertyGrid` (virtualized when needed)
  - “mixed value” affordances and reset-to-default affordances
- A small set of common editor controls:
  - `Checkbox` (bool + tri-state/mixed via `Option<bool>`)
  - `ColorEdit` (hex + swatch + minimal popup)
  - `VecNEdit`, `TransformEdit` (composites built on numeric primitives)
  - `EnumSelect` (filterable select)
  - `AssetRefField` (UI only; async state via optional query integration)
- A composable “panel recipe” layer (not an app framework):
  - `InspectorPanel`, `Palette` (search + list), small toolbars/headers

Explicitly out of scope (v1):

- New runtime contracts in `crates/fret-ui` (only add if evidence shows it is unavoidable).
- Domain-specific editors (node graphs, plots, timelines) as first-class widgets.
  - These will be integrated later by composing `fret-ui-editor` primitives with existing ecosystem crates
    (e.g. `ecosystem/fret-node`, `ecosystem/fret-plot`, `ecosystem/fret-chart`).

## Layering and dependencies

### Target dependency direction

`fret-ui-editor` is an ecosystem-level crate. It must remain renderer/platform agnostic and should not pull
domain crates as dependencies.

Allowed dependencies (typical):

- Required: `crates/fret-ui`, `ecosystem/fret-ui-kit`
- Recommended: `ecosystem/fret-ui-headless` (deterministic state machines / helpers)
- Optional:
  - `ecosystem/fret-undo` (undo coalescing semantics for scrub sessions)
  - `ecosystem/fret-selector` / `ecosystem/fret-query` (state adapters; feature-gated)
  - `ecosystem/fret-authoring` (only if we add `UiWriter`-based authoring helpers; feature-gated)

Forbidden dependencies:

- Any platform/runner/render crates: `fret-platform-*`, `fret-launch`, `fret-render*`, `wgpu`, `winit`, etc.
- Domain ecosystems as direct deps (`fret-node`, `fret-plot`, `fret-chart`, …). Integration should be via
composition in app crates or a dedicated “integration/recipes” crate.

### Skinning strategy (no shadcn hard dependency)

`fret-ui-editor` must **not** depend on `ecosystem/fret-ui-shadcn`. A separate optional crate may provide
a shadcn-aligned skin:

- `ecosystem/fret-ui-editor` (core editor controls; design-system agnostic)
- `ecosystem/fret-ui-editor-shadcn` (optional shadcn token/recipe adapters; depends on editor + shadcn)

This avoids long-term dependency cycles and keeps the editor surface reusable outside shadcn apps.

## Icon strategy (editor chrome)

`fret-ui-editor` uses **semantic icon IDs** from `ecosystem/fret-icons` for small chrome affordances
(e.g. disclosure chevrons, combo caret, clear buttons). Rendering is done via `SvgIcon` elements
(`crates/fret-ui` mechanism) and SVG sources resolved through the global icon registry (`IconRegistry`).

Guidelines:

- Components should prefer semantic IDs (`ui.chevron.down`, `ui.close`, …) rather than vendor IDs.
- Apps should install an icon pack (e.g. `ecosystem/fret-icons-lucide`) or provide their own mapping.
- The editor-proof demo installs the lucide pack to avoid “missing icon” placeholders.

## Authoring model (single source of truth)

Rules:

1) Declarative-first: core implementations should be authored via `ElementContext` + `AnyElement` / `ui()` builder.
2) No duplicate widget implementations: if an imui façade exists, it must delegate to the declarative implementation.
3) Keep patches centralized: use the unified patch chain (`UiBuilder<T>` / `ui()`) for layout/chrome decisions.

### Architecture direction (Plan A)

This workstream explicitly follows the “Plan A” architecture:

- **Single source of truth**: all widgets have one authoritative, declarative implementation.
- **imui is a façade**: immediate-style authoring is a frontend that mounts the same declarative widgets (no parallel runtime).
- **Policy stays in ecosystem**: “editor hand feel” (chrome defaults, hover intent, dismiss policy, focus restoration,
  density, icons, reset affordances) remains in `ecosystem/fret-ui-editor` and adjacent policy crates.
- **Mechanisms stay in `crates/fret-ui`**: layout, focus/capture, overlay mounting, painting, and accessibility contracts.

This is consistent with the already-landed imui v2 consolidation and the unified patch chain (ADR 0160).

### Visual baseline (v1)

For the proof demo to be usable, v1 also needs a minimal “visual baseline” policy:

- Interactive controls must render a stable **frame** (background, border, radius, padding) by default.
- “Bare text” surfaces are not acceptable for editable controls in the editor proof harness.
- Hidden/disabled internal subtrees must not paint or leak hit testing (clip/absolute or unmount when appropriate).
- Icons may start as glyph fallbacks, but public APIs should reserve icon slots for future skins.

### Public API naming rules (recommended)

Goals:

- Keep names discoverable for editor builders without binding to a specific upstream brand (ImGui/shadcn).
- Make it obvious which modules are stable vs still evolving.

Guidelines:

- Prefer “what it is” names: `DragValue`, `ColorEdit`, `PropertyGrid`, `InspectorPanel`.
- Avoid upstream brand names in the public API: do not expose `ImGui*` types in `fret-ui-editor` APIs.
- Use module prefixes to signal maturity:
  - `primitives::*`: shared, stability-oriented building blocks (prefer stable once adopted).
  - `controls::*`: individual controls (stabilize after they are used by editor-proof demos).
  - `composites::*`: composed recipes (may evolve; stabilize once apps depend on them).
  - `experimental::*`: explicitly unstable surfaces (allowed to churn).
- Keep “skin” naming out of the core crate (no `*-shadcn` modules in `fret-ui-editor`).

## State integration (optional, feature-gated)

`fret-ui-editor` should keep its core APIs usable without adopting a specific state model. State integration is
provided as optional glue behind feature flags, mirroring the pattern used by `fret-ui-shadcn`:

- `state-selector`: enables selector-based derived state helpers (`fret-selector/ui`)
- `state-query`: enables async query state helpers (`fret-query` and optionally `fret-query/ui`)
- `state`: umbrella for both

Guideline:

- Core components should not require `QueryState<T>` or selector types in their public constructors.
- Provide a `fret_ui_editor::state` module with small helper functions and adapter traits.

## Token strategy (`editor.*`)

The editor surface will introduce a small, namespaced token vocabulary so that:

- editor primitives can be themed consistently across the ecosystem, and
- other ecosystem crates (node/chart/plot) can opt into editor-grade density without hardcoding styles.

Principles:

- Use a stable namespace: `editor.*`
- Prefer **domain keys** over per-component explosion (keep the key set small and reusable).

Suggested v1 key families:

- `editor.density.*` (row height, padding, hit thickness, icon size)
- `editor.numeric.*` (scrub speed, slow/fast multipliers, precision defaults)
- `editor.property.*` (label/value column gap, label width policies, group header height)
- `editor.checkbox.*` (checkbox sizing/radius; colors should remain theme-driven)
- `editor.enum_select.*` (dropdown/list sizing; row density comes from `editor.density.*`)
- `editor.axis.*` (axis label colors for vec/transform controls)
- `editor.color.*` (swatch size, popup padding)

### Minimal v1 token table (proposed)

Notes:

- Metric tokens are expected to be resolved via `Theme::metric_by_key("...")` (or a theme alias) and interpreted as `Px`.
- Color tokens are expected to be resolved via `Theme::color_by_key("...")` (or a theme alias).
- Interaction booleans (e.g. “double-click to type”) should generally be settings (struct fields), not theme tokens.

| Token key | Type | Meaning | Default direction (non-normative) |
| --- | --- | --- | --- |
| `editor.density.row_height` | metric | Default inspector row height | near `component.list.row_height` |
| `editor.density.padding_x` | metric | Default horizontal padding for dense editor controls | small (tight) |
| `editor.density.padding_y` | metric | Default vertical padding for dense editor controls | small (tight) |
| `editor.density.hit_thickness` | metric | Minimum pointer target thickness (handles, scrub) | >= row_height * 0.6 |
| `editor.density.icon_size` | metric | Default icon size for toolbars/row actions | compact (14–16px equivalent) |
| `editor.numeric.scrub_speed` | metric | Base scrub delta per px | tuned per density |
| `editor.numeric.scrub_slow_multiplier` | metric | Shift slow mode multiplier | < 1.0 |
| `editor.numeric.scrub_fast_multiplier` | metric | Alt fast mode multiplier | > 1.0 |
| `editor.numeric.scrub_drag_threshold` | metric | Minimum drag distance (px) before scrubbing starts | small (2–6px) |
| `editor.numeric.error_fg` | color | Numeric input validation/error foreground | near `destructive` |
| `editor.property.column_gap` | metric | Label/value column gap | small |
| `editor.property.group_header_height` | metric | Collapsible group header height | row_height-ish |
| `editor.checkbox.size` | metric | Checkbox visual square size (inside hit target) | ~16px |
| `editor.checkbox.radius` | metric | Checkbox corner radius | small |
| `editor.enum_select.max_list_height` | metric | Max height for enum select list viewport | medium |
| `editor.axis.x_color` | color | Axis label color (X) | red-ish |
| `editor.axis.y_color` | color | Axis label color (Y) | green-ish |
| `editor.axis.z_color` | color | Axis label color (Z) | blue-ish |
| `editor.axis.w_color` | color | Axis label color (W) | muted |
| `editor.color.swatch_size` | metric | Color swatch square size | icon_size-ish |
| `editor.color.popup_padding` | metric | Picker popup padding | small/medium |

## Demo / repro

Proof harness (native):

- Run: `cargo run -p fret-demo --bin imui_editor_proof_demo`
- Single-window mode (no tear-off): `$env:FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW="1"; cargo run -p fret-demo --bin imui_editor_proof_demo`

## Interaction contracts (v1)

### Numeric edit session

We treat numeric editing as a first-class “edit session” concept with consistent outcomes:

- Start: pointer down (scrub) or enter typing mode (double-click or explicit action).
- Live updates: while scrubbing/typing, value updates are emitted.
- Commit: pointer up (scrub) or Enter/blur (typing).
- Cancel: Escape restores the pre-edit value.
- Undo coalescing: scrubbing should coalesce intermediate updates into a single history record (optional integration).

### Modifier semantics (recommended default)

Default outcome targets (subject to tuning via tokens):

- `Shift`: slow / precision mode (multiplier < 1.0)
- `Alt`: fast / coarse mode (multiplier > 1.0)

This matches common editor conventions and aligns with many ImGui-style workflows (non-normative reference:
`repo-ref/imgui/`).

## Stability policy (v1)

Although `fret-ui-editor` is an ecosystem crate and may iterate quickly, some behaviors become “sticky” once a
real inspector depends on them. We treat the following as “do not casually break” outcomes:

- Numeric edit session outcomes (commit/cancel semantics, pre-edit restore).
- Modifier semantics (Shift slow, Alt fast) as the default mapping (settings may override).
- Undo coalescing boundaries for scrub sessions (if undo integration is enabled).
- Focus/capture correctness (no accidental focus steals under overlays; pointer capture during scrub).

API tiers (recommended):

- **Stable**: primitives and controls used by the editor-proof demos and expected to be reused broadly.
- **Experimental**: new controls/composites; keep them in an `experimental` module or behind feature gates until proven.

## Regression gates

We expect these controls to become “sticky” behaviors. The workstream should leave behind:

- A runnable demo integration in an existing editor-proof harness (preferred: `imui_editor_proof_demo`).
- Scripted interaction repro(s) for the highest-risk controls:
  - `DragValue` (drag + modifiers + cancel/commit)
  - `PropertyGrid` (focus traversal + mixed values + reset)
- At least one constrained viewport/DPI variant for geometry-sensitive pieces (popup/picker).

### Run the proof demo

- Native: `cargo run -p fret-demo --bin imui_editor_proof_demo`
- Single-window degrade mode (PowerShell): `$env:FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW=\"1\"; cargo run -p fret-demo --bin imui_editor_proof_demo`
- Single-window degrade mode (POSIX): `FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW=1 cargo run -p fret-demo --bin imui_editor_proof_demo`

### `test_id` conventions (recommended)

Use stable, namespaced ids so scripted repros survive refactors:

- `editor.drag_value.<field>`
- `editor.numeric_input.<field>`
- `editor.property_row.<path>`
- `editor.property_group.<path>`
- `editor.color_edit.<field>`

Rule of thumb:

- Prefer semantic ids (field name / path) over positional ids.
- Treat ids referenced by scripts as stable API.

## ADR policy

This workstream is intentionally **not** an ADR. We only add or update ADRs if:

- a stable public runtime contract must change in `crates/fret-ui`, or
- we introduce a long-lived file format / persistence contract, or
- a cross-crate, hard-to-change behavior needs to be locked as a framework contract.

Otherwise, track decisions and progress in the TODO and alignment docs:

- `docs/workstreams/ui-editor-v1-todo.md`
- `docs/workstreams/ui-editor-imgui-alignment-v1.md`
