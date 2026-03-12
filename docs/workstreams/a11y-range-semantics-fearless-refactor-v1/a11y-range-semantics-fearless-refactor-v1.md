# A11y range/numeric semantics (fearless refactor v1)

Status: Shippable (core contract + AccessKit mapping + shadcn adoption + gates landed; contract locked in ADR 0288; follow-ups tracked below)

Last updated: 2026-02-24

## Motivation

Fret’s portable semantics tree (`crates/fret-core/src/semantics.rs`) currently exposes **roles**, **boolean flags**
(`disabled/selected/expanded/checked`), **relations** (`active_descendant`, `labelled_by`, `described_by`, `controls`),
and a **string** `label`/`value`. This is a solid skeleton, but it is insufficient for **range / numeric controls**
(slider/progress/scroll-like controls) where assistive technologies expect **structured numeric properties**
(`min/max/now/step`).

Symptoms before this workstream:

Update (2026-02-23): The core contract now includes structured numeric + scroll fields, the AccessKit adapter emits them
best-effort for finite values, `scroll_by` is wired end-to-end for scrollable host widgets, and sliders expose
Increment/Decrement stepper actions end-to-end (runner + default UI driver hooks).

Update (2026-02-23): Declarative `Pressable` nodes with `role=Slider` also expose stepper actions by default, and can
publish the structured numeric metadata via `SemanticsDecoration` (e.g. imui slider surfaces).

- A slider/progress often becomes “just text” (e.g. `"50%"`) rather than a role with `aria-valuenow/min/max`-like data.
- The diagnostics harness’s `set_slider_value` step must parse floats out of `SemanticsNode::value` strings
  (`ecosystem/fret-bootstrap/src/ui_diagnostics.rs`), which is brittle and locale-unfriendly.
- The AccessKit adapter maps roles but does not emit `numeric_value` / `min_numeric_value` / `max_numeric_value`
  (`crates/fret-a11y-accesskit/src/lib.rs`), even though AccessKit supports them.

This is already called out as an open contract gap in ADR 0181 (“Semantics value for range controls”).

## Scope note

This workstream is intentionally named for the **range/numeric** gap because it is the highest-ROI missing “main trunk”.
However, once we touch the semantics contract, it is worth auditing adjacent gaps that are similarly “mechanismizable”
(portable, structured, and map cleanly into AccessKit).

## Goals

- Add a **portable, structured numeric/range semantics surface** in `fret-core` (mechanism/contract layer).
- Map that surface into AccessKit’s numeric properties for native platforms.
- Update ecosystem components (shadcn/Radix-aligned slider/progress) to populate numeric semantics without encoding values
  into strings for correctness.
- Keep the change additive and migration-friendly: existing `label`/`value` strings remain valid.
- Tighten diagnostics + automation: prefer numeric semantics in scripts, keep a string fallback.

## Non-goals (v1)

- Perfect parity with every platform API and every ARIA attribute.
- A full “value formatting / localization” layer in the mechanism crates.
- Implementing every numeric-like widget (e.g. scrollbars, spinbuttons) in one PR.

## Ownership / layering

- `crates/fret-core`: the **portable contract** (structured semantics extras on `SemanticsNode`).
- `crates/fret-a11y-accesskit`: **adapter** (emit AccessKit numeric properties when present).
- `crates/fret-ui`: semantics snapshot production plumbing (forward fields from `SemanticsProps` / `SemanticsDecoration`).
- `ecosystem/*`: policy + recipes (shadcn, Radix-aligned composition decides what values/labels to expose).

## Landed contract shape (core)

Instead of adding many top-level optional fields onto `SemanticsNode`, we use an additive “extras” bucket:

- `SemanticsNode { .., extra: SemanticsNodeExtra, .. }`
- `SemanticsNodeExtra` currently contains:
  - `placeholder: Option<String>`
  - `url: Option<String>` (primarily for `SemanticsRole::Link`)
  - `level: Option<u32>` (1-based hierarchy level for outline/tree semantics)
  - `numeric: SemanticsNumeric { value/min/max/step/jump }`
  - `scroll: SemanticsScroll { x/x_min/x_max/y/y_min/y_max }`

Additional additive surfaces landed in the same refactor window:

- `SemanticsFlags::read_only` (portable text flag)
- `SemanticsFlags.checked_state` (tri-state checked: `false/true/mixed`, see ADR 0289)
- `SemanticsRole::Image` (portable role)

Notes:

- For indeterminate progress, omit `extra.numeric.value` and keep the role as `ProgressBar`.
- Keep `value: Option<String>` as a *human-readable* string (screen readers may still use it, diagnostics can display it).
- AccessKit mapping is best-effort: only emit numeric/scroll properties for finite values.
- Validation: `SemanticsNode::validate()` rejects non-finite numeric/scroll values; requires `min <= max` when both are present; requires `value` within `[min,max]` when all are present; requires positive `step/jump`; requires scroll positions within `[min,max]` when all are present; and enforces `level` as 1-based.
- See ADR 0288 for the contract rationale and invariants: `docs/adr/0288-a11y-numeric-and-range-semantics-v1.md`.

Action notes:

- `SemanticsActions.scroll_by` is the portable “scroll by delta” action surface.
- `SemanticsActions.increment` / `SemanticsActions.decrement` are a portable stepper surface for “range-like” controls
  (slider/spinbutton/splitter). In `fret-ui`, `value_editable` on these roles maps to `increment/decrement` (instead of
  always exposing `SetValue`).
- Range-like controls may also expose a `SetValue` surface; the default `fret-ui-app` driver implements it by
  translating target values into `Home/End/PageUp/PageDown/ArrowUp/ArrowDown` key sequences. This surface is gated by
  the runtime: it is only exposed when the numeric metadata includes `value/min/max/step`.

## Closure checklist (range/numeric controls)

Use this checklist when implementing or refactoring any “range-like” control (slider, progress, scrollbar-like,
spinbutton-like) to avoid drifting back into string-only semantics.

### 1) Contract + invariants (mechanism layer)

- [ ] Role is correct (`SemanticsRole::Slider` / `ProgressBar` / `Viewport` / etc.).
- [ ] Structured numeric metadata is published via `SemanticsNodeExtra.numeric`:
  - `value`: current value (`Some(f64)`), omitted for indeterminate states (e.g. indeterminate progress).
  - `min`/`max`: bounds when known; ensure `min <= max`.
  - `step`: positive step size when the control supports stepper semantics.
  - `jump`: optional “page” step (e.g. PageUp/PageDown), positive when present.
- [ ] All numeric fields are finite (no `NaN`/`±Inf`), so `SemanticsNode::validate()` passes.

### 2) Production (UI/runtime wiring)

- [ ] Values are emitted from a real element in the tree (not only from tests):
  - declarative widgets: use `SemanticsProps` or `AnyElement::attach_semantics(SemanticsDecoration)`.
  - retained widgets: publish via `Widget::semantics` / `SemanticsCx`.
- [ ] Prefer `SemanticsDecoration` when you only need a11y stamping on an existing typed element (avoid layout wrappers
  unless you truly need a semantics node boundary).
- [ ] For `Pressable`-based sliders:
  - `PressableA11y.role = Some(SemanticsRole::Slider)`.
  - Numeric metadata comes from `SemanticsDecoration` (or an explicit `SemanticsProps` wrapper), and stays in sync with
    the model.

### 3) Actions (adjustable controls)

- [ ] Range-like controls expose stepper actions (portable):
  - `SemanticsActions.increment` / `decrement` are enabled when the control is editable.
  - `invoke/click` is suppressed for stepper-like roles (avoid confusing AT).
- [ ] `SetValue` for range-like controls is only exposed when numeric metadata is sufficient:
  - `value + min + max + step` present (runtime-gated).
  - The platform action decoder and UI driver can deterministically apply the target value.

### 4) Adapter mapping (AccessKit)

- [ ] AccessKit mapping emits numeric fields (`numeric_value`, `min_numeric_value`, `max_numeric_value`, `step`, `jump`)
  best-effort for finite values.
- [ ] At least one adapter unit test covers the role + numeric fields (or documents a platform limitation).

### 5) Diagnostics + gates

- [ ] Diagnostics/automation prefers structured numeric semantics (no locale-dependent string parsing).
- [ ] Add at least one stable gate for each new production path:
  - unit/integration test asserting semantics snapshot fields, or
  - shadcn semantics snapshot JSON gate, or
  - a `fretboard diag` script that asserts numeric metadata.

Evidence anchors (examples of “done right”):

- shadcn slider/progress: `ecosystem/fret-ui-shadcn/src/slider.rs`, `ecosystem/fret-ui-shadcn/src/progress.rs`
- material3 slider: `ecosystem/fret-ui-material3/src/slider.rs`
- pressable slider actions: `crates/fret-ui/src/declarative/host_widget/semantics.rs`
- imui slider stamping: `ecosystem/fret-ui-kit/src/imui.rs`
- gate examples: `crates/fret-ui/src/declarative/tests/semantics.rs`, `ecosystem/fret-imui/src/lib.rs`

## Additional “mechanismizable” semantics gaps (candidates)

These are **not required** to ship the numeric/range backbone, but they are strong candidates to batch into the same
fearless refactor window if you want to avoid follow-up contract churn.

### A) Scroll semantics (high ROI for Viewport/Scroll/Scrollbar)

As of 2026-02-23, `SemanticsRole::Viewport` maps to AccessKit `ScrollView`. In practice, Fret publishes portable scroll
positions/ranges via `SemanticsNodeExtra.scroll` on scrollable host widgets (e.g. `Scroll`) when available. Portable
scroll properties enable both AT and automation to reason about scroll state.

Update (2026-02-23): Scroll containers also publish `extra.orientation` (horizontal/vertical) when the axis is known.

Candidate fields on `SemanticsNodeExtra.scroll`:

- `x`, `x_min`, `x_max`
- `y`, `y_min`, `y_max`

Candidate actions (portable):

- `scroll_by` (payload: dx/dy in logical units) — aligns with ADR 0033’s mention of `ScrollBy`.

### B) Tree/outline hierarchy level (for `TreeItem`, future `Heading`)

Update (2026-02-23): The portable `extra.level` field exists and maps to AccessKit `level`. The remaining work is
ecosystem adoption (populate levels for `TreeItem`, and later `Heading`).

We have `SemanticsRole::TreeItem`, but no portable “level” field, so platforms cannot announce hierarchy depth.

Candidate field:

- `level: Option<u32>` (map to AccessKit `level`)

### C) Text input flags and properties (read-only, placeholder)

We already have text selection/composition ranges (ADR 0071). Two common missing pieces:

- `read_only` flag (e.g. selectable but not editable text widgets)
- `placeholder` text (should map to AccessKit `placeholder` instead of being purely visual)

### D) Link + URL semantics (node-level, not only inline spans)

We can emit inline link spans today, but the AccessKit bridge does not translate them. A simpler incremental step is a
node-level URL for `SemanticsRole::Link`.

Candidate field:

- `url: Option<String>` (map to AccessKit `url`)

### E) Image semantics (alt text)

Fret has an `Image` element instance but no semantics role for it. If we want images to be discoverable by AT, add:

- `SemanticsRole::Image`
- `label` as “alt text”

## AccessKit mapping

When numeric fields are present, set them on the AccessKit node:

- `set_numeric_value`, `set_min_numeric_value`, `set_max_numeric_value`
- `set_numeric_value_step`, `set_numeric_value_jump`

This should apply primarily to roles like `Slider` and `ProgressBar`, but the mapping should be data-driven (if fields are
present, emit them).

## Ecosystem adoption (shadcn first)

Targets (first pass):

- Slider thumb nodes (`SemanticsRole::Slider`) should emit numeric value + min/max + step (+ jump if we decide).
  - Source: `ecosystem/fret-ui-kit/src/primitives/slider.rs` and `ecosystem/fret-ui-shadcn/src/slider.rs`
- Progress indicator nodes (`SemanticsRole::ProgressBar`) should emit numeric `now/min/max` when determinate.
  - Source: `ecosystem/fret-ui-shadcn/src/command.rs` loading row; and optionally `ecosystem/fret-ui-shadcn/src/progress.rs`
    (currently visual-only).

## Diagnostics + automation (gates)

Update the script engine to prefer structured numeric values:

- `SetSliderValue` should first read `SemanticsNode.extra.numeric.value` (when present), otherwise fallback to parsing
  `SemanticsNode.value` as today.

Regression protection:

- Unit tests in `crates/fret-a11y-accesskit` verifying numeric properties are emitted when present.
- Semantics snapshot tests in `ecosystem/fret-ui-shadcn` verifying slider/progress nodes expose numeric fields.
- Keep at least one `tools/diag-scripts/*set-value*.json` script passing.

## Evidence anchors (starting list)

- `crates/fret-core/src/semantics.rs`
- `crates/fret-ui/src/tree/ui_tree_semantics.rs`
- `crates/fret-a11y-accesskit/src/lib.rs`
- `ecosystem/fret-ui-kit/src/primitives/slider.rs`
- `ecosystem/fret-ui-shadcn/src/slider.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_slider.rs`
- `docs/adr/0181-ui-automation-and-debug-recipes-v1.md`

## Implementation snapshot (what’s landed)

- Core contract: `crates/fret-core/src/semantics.rs` (`SemanticsNodeExtra`, `SemanticsNumeric`, `SemanticsScroll`,
  `SemanticsFlags::read_only`, `SemanticsRole::Image`).
- Snapshot plumbing: `crates/fret-ui/src/tree/ui_tree_semantics.rs` forwards `extra` through `SemanticsCx`.
- Declarative + widgets:
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs` emits `Image` role and scroll positions for `Scroll`.
  - `crates/fret-ui/src/text/input/widget.rs` and `crates/fret-ui/src/text/area/widget.rs` emit placeholder.
- AccessKit adapter: `crates/fret-a11y-accesskit/src/lib.rs` maps numeric/scroll/placeholder/url/level/read-only.
- Ecosystem adoption:
  - `ecosystem/fret-ui-shadcn/src/slider.rs` emits numeric range/value.
  - `ecosystem/fret-ui-shadcn/src/progress.rs` emits determinate progress numeric semantics.
  - `ecosystem/fret-ui-material3/src/slider.rs` emits numeric range/value for Material 3 sliders.
- Diagnostics: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_slider.rs` prefers structured numeric semantics.

## Risks / mitigations

- **Breaking downstream code**: keep changes additive; default `None`; don’t repurpose `value`.
- **Locale formatting pitfalls**: store raw `f64` in semantics; formatting remains policy-layer.
- **API churn**: gate the contract in an ADR if needed; update `IMPLEMENTATION_ALIGNMENT.md` when implemented.
