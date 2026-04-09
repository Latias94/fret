# Generated Icon Presentation Defaults v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this file now records the shipped gate set for the closed lane.

## Smallest current repro

Use this sequence before changing the shipped presentation-defaults contract:

```bash
cargo nextest run -p fret-icons-generator -p fretboard
cargo nextest run -p fret-ui-kit --features icons -E 'package(fret-ui-kit) & (test(declarative::icon::tests::icon_authored_uses_svg_image_for_original_color_icons) | test(declarative::icon::tests::icon_authored_uses_svg_icon_for_mask_icons))'
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

What this proves now:

- generator/import proof still works,
- runtime authored-color vs mask-mode rendering remains explicit,
- and public docs/source-policy gates still teach the shipped config surface.

## Current evidence set

- `docs/workstreams/generated-icon-presentation-defaults-v1/BASELINE_AUDIT_2026-04-09.md`
  freezes the M0 baseline:
  - runtime already supports `OriginalColors`,
  - generated packs still default all imports to `Mask`,
  - generator intermediate data is presentation-blind today,
  - and acquisition metadata exists but is not yet a presentation policy.
- `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
  freezes the v1 direction:
  - explicit versioned presentation config as the source of truth,
  - generator library + thin CLI ownership,
  - pack-level default plus per-icon overrides,
  - and explicit-config proof before heuristics.
- `docs/workstreams/generated-icon-presentation-defaults-v1/M2_PROOF_SURFACE_2026-04-09.md`
  closes the proof surface on:
  - explicit generator/CLI config,
  - generated per-icon render-mode registration,
  - and preserved runtime `SvgIcon` / `SvgImage` behavior.
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  closes the lane on:
  - explicit versioned config,
  - generated registration + provenance,
  - runtime authored-color proof,
  - and public docs/source-policy teaching surfaces.
- `icon-system-extension-v1` is closed on the runtime icon mechanism and pack protocol.
- `iconify-acquisition-prestep-v1` is closed on explicit acquisition and local snapshot/provenance
  artifacts.

## Gate set

### Generator/import baseline

```bash
cargo nextest run -p fret-icons-generator -p fretboard
```

### Runtime presentation split baseline

```bash
cargo nextest run -p fret-ui-kit --features icons -E 'package(fret-ui-kit) & (test(declarative::icon::tests::icon_authored_uses_svg_image_for_original_color_icons) | test(declarative::icon::tests::icon_authored_uses_svg_icon_for_mask_icons))'
```

### Public docs / source-policy gate

```bash
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

## Evidence anchors

- `docs/workstreams/generated-icon-presentation-defaults-v1/DESIGN.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/TODO.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/MILESTONES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/presentation_defaults.rs`
- `crates/fret-icons-generator/src/svg_dir.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/help.rs`
- `ecosystem/fret/src/lib.rs`

## Reference posture

- The Iconify collection `palette` hint may be useful evidence, but it is not yet the frozen
  defaulting rule for generated packs.
- Keep this lane closed on generator/imported-pack presentation defaults, not on runtime or
  acquisition ownership.
