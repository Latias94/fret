# Baseline Audit — 2026-04-09

Status: accepted M0 evidence freeze

Related:

- `docs/workstreams/generated-icon-presentation-defaults-v1/DESIGN.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/TODO.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/MILESTONES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/svg_dir.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/acquire.rs`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- Iconify `IconifyInfo` docs: `https://iconify.design/docs/types/iconify-info.html`

## Purpose

Freeze the current baseline before changing generated-pack presentation defaults:

- verify what the shipped runtime icon contract already proves,
- isolate where generated packs currently lose authored-color defaults,
- and identify the smallest contract surface this follow-on actually owns.

## Findings

### 1. The core icon/runtime contract already supports explicit authored-color rendering

Evidence:

- `ecosystem/fret-icons/src/lib.rs` defines `IconPresentation` and `IconRenderMode::{Mask, OriginalColors}`.
- `ecosystem/fret-ui-kit/src/declarative/icon.rs` routes `icon_authored(...)` to `SvgImage` when a
  resolved icon carries `OriginalColors`.
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md` already closed the
  runtime split.

Consequence:

- this lane does not need a new runtime rendering mechanism;
- it only needs generated packs to feed the existing mechanism correctly.

### 2. Generated packs currently register all imported icons through the default mask-mode path

Evidence:

- `crates/fret-icons-generator/src/templates.rs` emits `reg.register_svg_bytes(...)` for every
  generated vendor icon.
- `ecosystem/fret-icons/src/lib.rs` makes `register_svg_bytes(...)` equivalent to
  `IconDefinition::new(...)`, which keeps the default `Mask` presentation.

Consequence:

- imported multicolor/authored-color assets keep their SVG bytes,
- but generated packs still default them to the themed icon posture.

### 3. The generator intermediate model is presentation-blind today

Evidence:

- `crates/fret-icons-generator/src/svg_dir.rs` defines `CollectedSvg` with only:
  - `icon_name`,
  - `source_relative_path`,
  - and `svg_bytes`.
- `crates/fret-icons-generator/src/iconify.rs` also emits only `CollectedSvg`.
- `crates/fret-icons-generator/src/contracts.rs` has no presentation-default policy surface yet.

Consequence:

- a correct fix likely needs generator-side metadata flow or config,
- not only a small template change.

### 4. Acquisition already records useful upstream hints, but those hints are not yet a presentation contract

Evidence:

- `crates/fretboard/src/icons/acquire.rs` records upstream collection metadata, including
  `collection_info.palette`, in the acquisition provenance sidecar.
- Iconify `IconifyInfo` defines `palette` as icon-set-level metadata:
  - `true` when all icons use hardcoded colors,
  - `false` when all icons use `currentColor`.
- generated pack import does not currently read that provenance.
- local SVG directory imports have no collection-level metadata at all.

Consequence:

- upstream hints may inform a future policy,
- but they are not enough by themselves to define a stable default across all source kinds.

### 5. The lane owns generated-pack presentation policy, not acquisition or semantic policy

Evidence:

- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md` already closes
  acquisition on explicit local snapshot + provenance artifacts.
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md` already closes the
  icon runtime and pack protocol.

Consequence:

- this lane should freeze how generated/imported packs choose default presentation;
- it should not reopen remote acquisition, runtime split, or semantic alias policy.

## M0 decision from this audit

Treat M0 as closed on these points:

1. runtime authored-color support already exists and must stay unchanged here;
2. generated packs currently default all imports to `Mask`;
3. the missing contract is generator/import-side presentation policy and metadata flow;
4. the next freeze must decide whether the source of truth is explicit config, source analysis,
   upstream hints, or a layered combination.
