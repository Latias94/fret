# UI Gallery exemplar and evidence workflow

Use this note when shadcn parity work is really about how first-party Fret code should be authored,
taught, or proven.

## 1) Treat UI Gallery as the first-party exemplar surface

For first-party shadcn examples, the UI Gallery is not just a demo app. It is the in-tree teaching
surface that future docs, diagnostics, and app authors will copy from.

Prefer to audit these layers in order:

- `src/ui/snippets/**`: canonical example source
- `src/ui/pages/**`: gallery page composition around snippets
- `src/driver/**`: diagnostics/runtime glue, geometry helpers, chrome

Snippet files are the highest-signal source because they serve two roles:

- compiled preview
- copyable code tab via `include_str!`

If a snippet teaches the wrong import or composition pattern, fix the snippet first.
If the preview looks wrong, compare the snippet against the exact upstream example file before
touching the recipe: example-local `size`, `variant`, wrapper layout, and slot-local classes are
part of the teaching-surface truth too.

## 2) Authoring surface rules after the facade refactor

Do not mix app-facing `fret` examples with direct `fret_ui_shadcn` examples.

Recommended first-party direct-crate imports:

```rust
use fret_ui_shadcn::{facade as shadcn, prelude::*};
```

Recommended escape hatches:

```rust
use shadcn::raw::breadcrumb::primitives as bc;
// or other explicit `shadcn::raw::*` modules when the seam is intentionally non-curated
```

Guidance:

- Use the curated facade for first-party direct-crate samples.
- Keep `shadcn::raw::*` explicit so readers can see they are leaving the curated surface.
- Do not reintroduce `use fret_ui_shadcn as shadcn;` or `use fret_ui_shadcn::{self as shadcn, ...};`.
- For app-facing starter/documentation surfaces, align with the current `fret` guidance in
  `docs/crate-usage-guide.md` instead of copying direct-crate examples blindly.
- For widgets that already expose stable action slots, prefer `.action(...)` / `.action_payload(...)`
  over legacy `.on_click(...)` spelling on copyable first-party snippets.
- For activation-only surfaces inside first-party snippets, prefer `fret::view::AppActivateExt`
  (`.dispatch::<A>(cx)` / `.dispatch_payload::<A>(cx, payload)` / `.listen(cx, ...)`) rather than
  reopening raw `.on_activate(...)`; this stays valid in both `AppUi` and extracted `UiCx`
  helper functions.
- Keep copyable first-party snippets aligned with the active authoring target state:
  - app-facing teaching snippets/pages prefer `Ui`, `UiChild`, and `UiCx`,
  - generic reusable snippet helpers should converge on the unified component conversion trait
    tracked in `docs/workstreams/into-element-surface-fearless-refactor-v1/`,
  - advanced/manual-assembly reusable helpers should prefer `IntoUiElement<H>` rather than
    child-pipeline traits such as `UiChildIntoElement<H>`,
  - page/document wrapper seams should keep typed preview inputs typed for as long as possible;
    prefer wrapper entry points such as `DocSection::build(cx, title, preview)` when the wrapper
    itself can own the final landing,
  - prose/doc-note helpers should also stay typed until the page wrapper seam; prefer
    `doc_layout::notes_block(...)` over reintroducing `AnyElement`-returning doc helpers,
  - `AnyElement` stays for explicit raw/diagnostics/helper seams only,
  - do not eagerly call `.into_element(cx)` just to cross a docs/page scaffold boundary unless the
    wrapper is intentionally a raw seam that truly owns a concrete `AnyElement` contract,
  - do not teach the legacy split conversion trait names in copyable snippet tabs unless the
    example is explicitly documenting a raw/advanced seam.

## 3) Stable automation requires stable `test_id`

Before writing diag scripts or geometry assertions, ensure the sample exposes stable `test_id`
surfaces for:

- trigger/root/content anchors
- scroll/viewports when scrolling or clipping matters
- row/item prefixes when collections are under test
- optional “marker” nodes for action results or hover/focus state confirmation

If a parity fix cannot be tested without brittle selectors, add or rename `test_id` first.

## 4) Evidence ladder: prefer deterministic proof before screenshot churn

Use the smallest artifact that proves the regression:

1. Source-policy tests:
   - `apps/fret-ui-gallery/src/lib.rs`
   - Good for import discipline, allowed escape hatches, and authoring-surface drift.
2. Geometry/layout assertions:
   - `apps/fret-ui-gallery/src/driver/render_flow.rs`
   - Good for stable bounds, centering, fill/grow, clipping, and viewport calculations.
3. Layout sidecar dumps:
   - `capture_layout_sidecar`
   - Good when the dispute is “which node owns width/height/flex/min-size?” rather than colors.
4. Screenshots:
   - `capture_screenshot`
   - Good for visual chrome, clipping, hover/focus appearance, and constrained viewport checks.
5. Bundles:
   - `capture_bundle`
   - Good for preserving the final interaction/a11y/runtime evidence trail.

In practice:

- layout drift first: geometry assertions or layout sidecar
- visual drift next: screenshot
- interaction drift always: bundle, usually with stable `test_id`

## 5) When to use layout sidecar vs screenshot

Prefer `capture_layout_sidecar` when:

- the bug is about `w_full`, `flex_1`, `min_w_0`, stretch/shrink ownership, or slot negotiation
- two variants look similar visually but one has the wrong owner/ancestor in the layout tree
- you need reviewable structure that will survive theme/token churn

Prefer `capture_screenshot` when:

- the bug is about visible chrome, focus ring, shadow, clipping, border, spacing, or truncation
- the expected outcome is easier to review visually than numerically

Use both when a layout dispute also affects visible clipping or overflow.

## 6) High-signal repo anchors

- Authoring policy gates: `apps/fret-ui-gallery/src/lib.rs`
- Action-surface policy gates: `tools/gate_button_action_default_surfaces.py`,
  `tools/gate_gallery_action_alias_default_surfaces.py`
- Snippet source of truth: `apps/fret-ui-gallery/src/ui/snippets/`
- Activation-only sugar owner: `ecosystem/fret/src/view.rs`
- Geometry helpers and prepared-frame wiring: `apps/fret-ui-gallery/src/driver/render_flow.rs`
- Diag capture implementation: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
- Script corpus examples: `tools/diag-scripts/ui-gallery/`
- Canonical shadcn migration guidance: `docs/shadcn-declarative-progress.md`
- Crate/layer selection guidance: `docs/crate-usage-guide.md`
- Into-element conversion cleanup: `docs/workstreams/into-element-surface-fearless-refactor-v1/`
