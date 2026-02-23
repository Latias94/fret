# Length percentage semantics v1 (percent / fraction closure)

Status: In progress (workstream)

Last updated: 2026-02-23

## Motivation

shadcn/ui + Radix-style recipes routinely depend on **percent sizing**:

- `basis-full` (carousel items are 100% of the viewport width)
- `w-full` / `h-full` (overlay roots, scroll viewports, pointer regions)
- `inset-0` style positioning (full-bleed overlays)

In a custom renderer with a Taffy-powered layout engine, percent sizing is easy to *support* but easy to get *wrong*:
if percent values are resolved under intrinsic / shrink-wrap measurement (min-content / max-content / auto containing block),
they can collapse to `0` and cause hard-to-debug clipping and overlap issues.

This workstream defines and closes Fret’s **percent/fraction** semantics end-to-end:

- mechanism (`crates/fret-ui`) percent resolution rules
- authoring (`ecosystem/fret-ui-kit`) fluent API surface
- ecosystem migration (`ecosystem/fret-ui-shadcn`) removing ad-hoc px clamps and workarounds
- regression gates (unit tests + diag scripts)

## Scope note (layering)

Percent sizing semantics are a **mechanism contract**:

- `crates/fret-ui`: defines the semantics and resolves them safely under measurement/layout passes
- `ecosystem/fret-ui-kit`: exposes authoring shorthands (`*_percent` / `*_fraction`) and keeps naming consistent
- `ecosystem/fret-ui-shadcn`: uses the semantics (should not patch around incorrect percent behavior)

Apps (UI gallery) should only provide repro/gate surfaces, not define semantics.

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed/GPUI: percent sizing via `DefiniteLength::Fraction` and `Length::{Definite,Auto}`
  - `repo-ref/zed/crates/gpui/src/geometry.rs`
  - `repo-ref/zed/crates/gpui/src/taffy.rs`
- CSS percent sizing mental model (informal reference): “percent requires a definite containing block; otherwise behaves like auto”

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

## Terms

### Fraction

Fret uses a ratio representation:

- `Fraction(0.5)` means 50% of the containing block size for that axis.
- `Fill` is a shorthand for 100% (equivalent intent to `Fraction(1.0)`).

### Definite containing block

Percent sizing only resolves when the containing block axis is **definite** (a real pixel extent is known).
During intrinsic sizing probes (min/max-content), an axis is *not* definite.

### Available space vs known dimensions

Taffy passes both:

- known dimensions: explicit constraints (`known.width/height`)
- available space: “what the parent can offer” (definite px, or min/max-content probes)

Percent sizing must be sensitive to **available space definiteness** to avoid collapse.

## Semantics (v1 policy)

Percent/fraction values are resolved with the following rule:

- If the available space for the axis is **definite**, resolve to px.
- Otherwise (min-content / max-content / auto-probing), treat percent/fraction as **auto**.

This is intentionally CSS-like: percent needs a definite containing block.
It also matches GPUI’s practical behavior: percent is meaningful when the parent provides a real extent, not during intrinsic probing.

### Clamp policy

For v1, percent/fraction resolution should be robust to invalid input:

- non-finite ratios behave like `0`
- negative ratios clamp to `0`
- ratios greater than `1.0` are allowed (e.g. 120%) unless a specific component opts into clamping

## Current implementation status (what is already landed)

| Area | Layer | Status | Evidence anchor |
|---|---|---:|---|
| Core representation: `Length::Fraction(f32)` | `crates/fret-ui` | Landed | `crates/fret-ui/src/element.rs` |
| Authoring: `w_fraction/w_percent`, `h_fraction/h_percent`, `basis_fraction/basis_percent` | `fret-ui-kit` | Landed | `ecosystem/fret-ui-kit/src/style/layout.rs` |
| Available-aware resolution (`Fill`/`Fraction` → px only when definite, else auto) | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/host_widget/measure.rs` |
| Wrapper-chain promotion for percent descendants | `crates/fret-ui` | Landed | `crates/fret-ui/src/layout/engine/flow.rs` |
| Regression tests (measurement + flex basis) | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/tests/layout/basics.rs` |

## Remaining closures (what is still missing)

Percent/fraction semantics are currently “closed” for `size` and `flex-basis` but not for the full authoring language.
The highest-ROI remaining work is to cover **spacing and constraints** consistently:

1. Size constraints
   - `min_width/min_height/max_width/max_height` should accept percent/fraction (with the same resolution rules).
2. Spacing
   - `padding` and `gap` should accept percent/fraction (definite-only, no `auto`).
   - `margin` and `inset` should accept percent/fraction (and preserve `auto` where it is meaningful).
3. Positioning + overlays
   - `inset` percent should enable “inset by % of containing block” outcomes without per-component math.
4. Migration
   - remove ad-hoc workarounds like “clamp percent basis to px when viewport extent is measurable” from recipes.

## Migration plan (incremental)

1. Extend the authoring language in `fret-ui-kit` (new `*_percent` / `*_fraction` shorthands) without changing existing callsites.
2. Teach the declarative bridge / layout engine to resolve the new percent-bearing fields using the v1 rule.
3. Migrate shadcn recipes one component at a time (start with the ones that match upstream docs: carousel, overlays, sheets/drawers).
4. Add one focused gate per migration (unit test when possible; diag script when it’s a UI outcome).

## Quality gates (v1)

- Formatting: `cargo fmt`
- Focused tests:
  - `cargo nextest run -p fret-ui -E "test(fraction_only_resolves_under_definite_available_space_in_measurement)"`
  - `cargo nextest run -p fret-ui -E "test(flex_fraction_basis_and_fill_basis_do_not_collapse_under_min_content_measurement)"`
- Targeted diag scripts (UI gallery):
  - Carousel basic screenshot gate: `tools/diag-scripts/ui-gallery-carousel-basic-screenshot.json`

