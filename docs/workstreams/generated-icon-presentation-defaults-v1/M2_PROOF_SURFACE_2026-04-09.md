# M2 Proof Surface — 2026-04-09

Status: accepted proof

Related:

- `docs/workstreams/generated-icon-presentation-defaults-v1/DESIGN.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/TODO.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/MILESTONES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/EVIDENCE_AND_GATES.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/presentation_defaults.rs`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/svg_dir.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`

## Purpose

Freeze the first real proof slice for the generated-pack presentation-defaults contract. This note
records the smallest landed evidence that:

- generated packs can explicitly register authored-color imports as `OriginalColors`,
- monochrome imports still stay on `Mask`,
- and the proof works without reopening runtime ownership or introducing heuristic defaulting.

## What shipped in the proof

### 1) Explicit versioned presentation-defaults config

`fret-icons-generator` now owns a public, versioned presentation-defaults contract:

- `PresentationDefaultsConfigFileV1`
- `PresentationDefaults`
- `PresentationOverride`
- `PresentationRenderMode`

The config supports:

- an optional pack-level `default_render_mode`,
- explicit per-icon overrides keyed by generated icon name,
- duplicate/empty override validation,
- and schema-version rejection for forward compatibility.

### 2) Thin public CLI wiring

`fretboard icons import svg-dir ...` and
`fretboard icons import iconify-collection ...` now both accept:

- `--presentation-defaults ./presentation-defaults.json`

The CLI remains thin:

- it loads the versioned JSON file,
- passes the resulting contract to `fret-icons-generator`,
- and does not move presentation policy into runtime or app layers.

### 3) Generated code and provenance now keep presentation explicit

Generated pack output now records the chosen presentation in two places:

- `src/lib.rs` registers each vendored icon with explicit `IconPresentation` /
  `IconRenderMode`,
- `pack-provenance.json` records both pack-level presentation defaults and each generated icon's
  resolved `render_mode`.

This keeps review-visible generated artifacts aligned with the frozen M1 decision.

### 4) Runtime authored-color path remains the existing one

The proof did not add a new runtime mechanism.

Instead it reuses the already-shipped split:

- `IconRenderMode::Mask` continues through `SvgIcon`,
- `IconRenderMode::OriginalColors` continues through `SvgImage`,
- and `icon_authored(...)` remains the author-facing helper that honors the explicit registry
  presentation.

## Proof gates executed on 2026-04-09

```bash
cargo nextest run -p fret-icons-generator -p fretboard
cargo nextest run -p fret-ui-kit --features icons -E 'package(fret-ui-kit) & (test(declarative::icon::tests::icon_authored_uses_svg_image_for_original_color_icons) | test(declarative::icon::tests::icon_authored_uses_svg_icon_for_mask_icons))'
```

Observed result:

- `fret-icons-generator` and `fretboard`: `74 tests run: 74 passed`
- `fret-ui-kit` authored-icon split gate: `2 tests run: 2 passed`

## Evidence anchors from the proof

- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/presentation_defaults.rs`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/svg_dir.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`

## M2 verdict

Treat M2 as closed on these points:

1. generated/imported packs now have an explicit, versioned presentation-defaults contract;
2. authored-color and mask-mode defaults are preserved in generated code and provenance;
3. the runtime split is proven unchanged and correctly reused;
4. future heuristic helpers, if any, are follow-ons rather than prerequisites for the shipped
   contract.

