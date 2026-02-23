# A11y range/numeric semantics (fearless refactor v1)

Status: In progress (partially landed)

Last updated: 2026-02-23

## Motivation

Fret’s portable semantics tree (`crates/fret-core/src/semantics.rs`) currently exposes **roles**, **boolean flags**
(`disabled/selected/expanded/checked`), **relations** (`active_descendant`, `labelled_by`, `described_by`, `controls`),
and a **string** `label`/`value`. This is a solid skeleton, but it is insufficient for **range / numeric controls**
(slider/progress/scroll-like controls) where assistive technologies expect **structured numeric properties**
(`min/max/now/step`).

Symptoms before this workstream:

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
- `SemanticsRole::Image` (portable role)

Notes:

- For indeterminate progress, omit `extra.numeric.value` and keep the role as `ProgressBar`.
- Keep `value: Option<String>` as a *human-readable* string (screen readers may still use it, diagnostics can display it).
- AccessKit mapping is best-effort: only emit numeric/scroll properties for finite values.
- Validation: `SemanticsNode::validate()` rejects non-finite numeric/scroll values; requires `min <= max` when both are present; requires `value` within `[min,max]` when all are present; requires positive `step/jump`; requires scroll positions within `[min,max]` when all are present; and enforces `level` as 1-based.

## Additional “mechanismizable” semantics gaps (candidates)

These are **not required** to ship the numeric/range backbone, but they are strong candidates to batch into the same
fearless refactor window if you want to avoid follow-up contract churn.

### A) Scroll semantics (high ROI for Viewport/Scroll/Scrollbar)

Today, `Viewport` maps to AccessKit `ScrollView`, but we do not emit scroll positions/ranges. Adding portable scroll
properties enables both AT and automation to reason about scroll state.

Candidate fields on `SemanticsNodeExtra.scroll`:

- `x`, `x_min`, `x_max`
- `y`, `y_min`, `y_max`

Candidate actions (portable):

- `scroll_by` (payload: dx/dy in logical units) — aligns with ADR 0033’s mention of `ScrollBy`.

### B) Tree/outline hierarchy level (for `TreeItem`, future `Heading`)

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
- Diagnostics: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_slider.rs` prefers structured numeric semantics.

## Risks / mitigations

- **Breaking downstream code**: keep changes additive; default `None`; don’t repurpose `value`.
- **Locale formatting pitfalls**: store raw `f64` in semantics; formatting remains policy-layer.
- **API churn**: gate the contract in an ADR if needed; update `IMPLEMENTATION_ALIGNMENT.md` when implemented.
