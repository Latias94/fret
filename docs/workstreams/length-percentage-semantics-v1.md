# Length percentage semantics v1 (percent / fraction closure)

Status: Shippable (percent/fraction semantics closed; shadcn migrations landed; gates passing)

Last updated: 2026-02-25

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

### Spacing (padding + gap)

For v1, percent-bearing spacing is treated as a *definite-only* contract:

- `padding` percent/fraction resolves against the containing block **width** (CSS-like), even for top/bottom.
- `gap` percent/fraction resolves against the **inner** available space (after padding shrink), since the gap lives inside the padding box.
- During intrinsic probes (min/max-content measurement), percent-bearing spacing resolves to `0px` (effectively ignored) to avoid cyclic dependencies.

## Current implementation status (landed)

| Area | Layer | Status | Evidence anchor |
|---|---|---:|---|
| Core representation: `Length::Fraction(f32)` | `crates/fret-ui` | Landed | `crates/fret-ui/src/element.rs` |
| Authoring: `w_fraction/w_percent`, `h_fraction/h_percent`, `basis_fraction/basis_percent` | `fret-ui-kit` | Landed | `ecosystem/fret-ui-kit/src/style/layout.rs` |
| Available-aware resolution (`Fill`/`Fraction` → px only when definite, else auto) | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/host_widget/measure.rs` |
| Wrapper-chain promotion for percent descendants | `crates/fret-ui` | Landed | `crates/fret-ui/src/layout/engine/flow.rs` |
| Regression tests (measurement + flex basis) | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/tests/layout/basics.rs` |
| Size constraints: `min_*` / `max_*` accept percent/fraction | `crates/fret-ui` + `fret-ui-kit` | Landed | `crates/fret-ui/src/element.rs` + `ecosystem/fret-ui-kit/src/style/layout.rs` |
| Spacing: percent-capable `padding` + `gap` (definite-only) | `crates/fret-ui` | Landed | `crates/fret-ui/src/element.rs` + `crates/fret-ui/src/declarative/host_widget/measure.rs` |
| Spacing authoring shorthands (`padding_percent`, `gap_percent`, ...) | `fret-ui-kit` | Landed | `ecosystem/fret-ui-kit/src/ui_builder.rs` |
| Spacing regression test (definite vs intrinsic measurement) | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/tests/layout/basics.rs` |
| Positioning: percent-capable `inset` + `margin` semantics | `crates/fret-ui` + `fret-ui-kit` | Landed | `crates/fret-ui/src/element.rs` + `ecosystem/fret-ui-kit/src/style/chrome.rs` |
| Positioning regression tests (inset + margin) | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/tests/layout/basics.rs` |
| shadcn migration: carousel “basis-full” no longer needs px clamps | `fret-ui-shadcn` | Landed | `ecosystem/fret-ui-shadcn/src/carousel.rs` |
| shadcn migration: sheet + drawer max-height clamps use fraction/fill | `fret-ui-shadcn` | Landed | `ecosystem/fret-ui-shadcn/src/sheet.rs` + `ecosystem/fret-ui-shadcn/src/drawer.rs` |
| Evidence (UI): carousel screenshot gate | `tools/diag-scripts` | Landed | `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-basic-screenshot.json` |
| Evidence (UI): sheet escape + focus-restore gate | `tools/diag-scripts` | Landed | `tools/diag-scripts/ui-gallery-sheet-escape-focus-restore.json` |
| Evidence (UI): drawer docs smoke gate | `tools/diag-scripts` | Landed | `tools/diag-scripts/ui-gallery-drawer-docs-smoke.json` |

## Closure notes

This workstream is considered **closed** for v1: percent/fraction semantics are supported end-to-end across:

- core sizing + flex-basis
- min/max constraints
- spacing (padding + gap)
- positioning (inset + margin)
- shadcn recipe migrations for the originally reported “percent collapse” symptoms

Follow-ups that are *out of scope* for this workstream should be tracked in their owning workstreams (e.g. Embla carousel interaction parity, overlay placement policy, or container-query fallback ergonomics).

## Quality gates (v1)

- Formatting: `cargo fmt`
- Focused tests:
  - `cargo nextest run -p fret-ui -E "test(fraction_only_resolves_under_definite_available_space_in_measurement)"`
  - `cargo nextest run -p fret-ui -E "test(flex_fraction_basis_and_fill_basis_do_not_collapse_under_min_content_measurement)"`
  - `cargo nextest run -p fret-ui -E "test(spacing_fraction_only_resolve_under_definite_available_space_in_measurement)"`
- Targeted diag scripts (UI gallery):
  - Carousel basic screenshot gate:
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-basic-screenshot.json --warmup-frames 5 --exit-after-run --launch -- cargo run -p fret-ui-gallery --release`
  - Sheet escape + focus-restore gate:
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-sheet-escape-focus-restore.json --warmup-frames 5 --exit-after-run --launch -- cargo run -p fret-ui-gallery --release`
  - Drawer docs smoke gate:
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-drawer-docs-smoke.json --warmup-frames 5 --exit-after-run --launch -- cargo run -p fret-ui-gallery --release`

### Optional: view-cache reuse sanity gate

If you want a “did we actually reuse cached subtrees?” check, run:

- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-view-cache-toggle-perf.json --warmup-frames 5 --check-view-cache-reuse-min 1 --timeout-ms 240000 --exit-after-run --launch -- cargo run -p fret-ui-gallery --release`

Notes:

- View-cache reuse gates depend on cache-root debug records. If you launch UI gallery manually outside of `fretboard`, also set `FRET_UI_DEBUG_STATS=1` so bundles include `debug.cache_roots`.
