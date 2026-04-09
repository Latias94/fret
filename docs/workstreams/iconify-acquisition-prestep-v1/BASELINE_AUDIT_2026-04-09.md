# Baseline Audit — 2026-04-09

Status: accepted M0 evidence freeze

Related:

- `docs/workstreams/iconify-acquisition-prestep-v1/DESIGN.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/TODO.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/MILESTONES.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/mod.rs`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `repo-ref/dioxus-iconify/README.md`
- `repo-ref/dioxus-iconify/src/api.rs`

## Purpose

Freeze the current acquisition baseline before widening the lane:

- verify what the shipped icon/generator contract already proves,
- separate “multicolor icon support” from “remote Iconify acquisition,”
- verify whether third-party developers already have enough public surface to build
  Iconify-like import tooling,
- and identify the narrow contract debt that still belongs to this follow-on.

## Findings

### 1. The shipped core icon contract already supports multicolor icons

Evidence:

- `ecosystem/fret-icons/src/lib.rs` defines `IconDefinition`, `IconPresentation`, and
  `IconRenderMode::{Mask, OriginalColors}`.
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md` already closed the
  runtime split on themed `SvgIcon` vs authored-color `SvgImage`.
- `ecosystem/fret-ui-kit/src/declarative/icon.rs` routes `icon_authored(...)` to `SvgImage` when
  an icon resolves to `OriginalColors`.

Consequence:

- this lane does not need to invent a new core multicolor mechanism;
- the underlying runtime/registry contract is already capable of preserving authored colors.

### 2. The current Iconify generator path preserves multicolor SVG data, but does not yet expose authored-color presentation

Evidence:

- `crates/fret-icons-generator/src/iconify.rs` preserves Iconify `body` content when building the
  emitted SVG, and `iconify_collection_preserves_multicolor_svg_body()` proves that authored
  `fill` values survive import.
- `crates/fret-icons-generator/src/templates.rs` currently registers generated icons with
  `reg.register_svg_bytes(...)`, which uses the default `Mask` presentation.
- `ecosystem/fret-ui-kit/src/declarative/icon.rs` proves that `OriginalColors` requires explicit
  presentation metadata to reach the authored `SvgImage` path.

Consequence:

- it is correct to say that current imports preserve multicolor assets;
- it is not yet correct to say that generated Iconify packs render in authored colors by default;
- any future “palette icon defaults” work is a pack/generator follow-on, not acquisition scope.

### 3. Third-party developers already have a real public surface for local-input pack generation

Evidence:

- `crates/fret-icons-generator/src/contracts.rs` exposes `GeneratePackRequest`, `SourceSpec`,
  `SvgDirectorySource`, `IconifyCollectionSource`, and versioned semantic alias config types.
- `crates/fret-icons-generator/src/lib.rs` exposes `generate_pack_crate(...)` as a reusable
  library entrypoint.
- `crates/fretboard/src/icons/mod.rs` is now a thin public CLI wrapper over that reusable library.
- `ecosystem/fret-icons/src/lib.rs` defines the explicit pack registration contract:
  `IconPackMetadata`, `IconPackRegistration`, and `InstalledIconPacks`.
- `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md` already closed
  the library + thin CLI ownership decision.

Consequence:

- external developers can already build “Iconify-like” tooling so long as they produce local input
  for the generator or call the generator library directly;
- this repo no longer has a hard dependency on first-party `tools/` scripts for third-party pack
  generation;
- the remaining missing surface is remote acquisition, not pack emission.

### 4. The real contract gap is pinned remote acquisition and provenance recording

Evidence:

- the closed generator lane explicitly forbids hiding network fetch inside
  `fretboard icons import ...`;
- `crates/fret-icons-generator/src/contracts.rs` models only local sources today;
- generated `pack-provenance.json` records pack-generation facts, not remote acquisition facts;
- `repo-ref/dioxus-iconify/src/api.rs` shows one possible HTTP acquisition client, but it does not
  define Fret's artifact or provenance contract.

Consequence:

- this follow-on should freeze:
  - how remote Iconify data becomes local input,
  - what pinning/provenance must be written down,
  - and whether the acquisition result is one file or multiple files;
- this follow-on should not redesign the emitted pack crate, app install seams, or semantic alias
  config.

### 5. `repo-ref/dioxus-iconify` is useful for workflow inspiration, but not for contract import

What is useful:

- a separate CLI-focused acquisition workflow,
- build-time vendoring posture,
- and user ergonomics around explicit icon collection selection.

What does not transfer directly:

- its runtime/framework-specific generated code shape,
- its “add icons straight into the app source tree” posture,
- and treating a live HTTP client as the normative public boundary.

Consequence:

- use `repo-ref/dioxus-iconify` as a UX/process reference only;
- keep Fret's normative contract centered on pinned local artifacts plus the existing pack
  generator.

## M0 decision from this audit

Treat M0 as closed on these points:

1. core multicolor support already exists and must not be reopened here;
2. generated Iconify packs preserve multicolor SVG bytes, but presentation defaults are a separate
   concern from acquisition;
3. third-party local-input pack generation is already a real public contract;
4. the only contract this lane still owns is explicit remote acquisition of pinned local artifacts
   plus provenance.

## Guidance for M1

The next freeze should decide:

1. whether acquisition emits:
   - a full Iconify collection snapshot,
   - a subset snapshot in the same Iconify collection schema,
   - or a snapshot plus explicit provenance sidecar;
2. whether acquisition lives as:
   - a `fretboard` command only,
   - or a reusable acquisition helper plus thin CLI;
3. which provenance facts are mandatory at acquisition time:
   - source URL/template,
   - collection prefix,
   - requested icons or subset definition,
   - and any license/palette metadata that should remain reviewable even if the generator ignores it.

Initial recommendation:

- preserve generator input compatibility by emitting an Iconify-collection-shaped local snapshot;
- prefer a separate provenance sidecar rather than hiding acquisition-only facts inside pack output;
- and keep acquisition explicitly two-step instead of folding it into `icons import`.
