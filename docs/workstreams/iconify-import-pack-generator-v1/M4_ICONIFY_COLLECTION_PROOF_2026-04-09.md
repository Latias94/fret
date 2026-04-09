# M4 Iconify Collection Proof — 2026-04-09

Status: landed follow-on proof slice

Status note (2026-04-09): this remains the source-expansion proof read for local Iconify
collection snapshots. The final shipped verdict for the full lane now lives in
`docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`.

Related:

- `docs/workstreams/iconify-import-pack-generator-v1/DESIGN.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/TODO.md`
- `docs/workstreams/iconify-import-pack-generator-v1/MILESTONES.md`
- `docs/workstreams/iconify-import-pack-generator-v1/EVIDENCE_AND_GATES.md`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`

## Purpose

This note records the first landed M4 source-expansion slice for
`iconify-import-pack-generator-v1`.

The goal stayed intentionally narrow:

- do not reopen the frozen producer contract,
- do not add a runtime Iconify client or live network fetch path,
- and do not mix semantic alias policy into the generator by default.

Instead, land the smallest follow-on that proves the reusable generator surface can accept a local
Iconify collection snapshot and still emit the same generated pack-crate contract already proven by
the SVG-directory path.

## Commands used

Generator/library + CLI proof:

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo check -p fretboard --quiet`

Existing shipped icon-pack contract regression checks:

- `cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix`
- `cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"`

User-facing doc guards:

- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`

## Result summary

### 1) The reusable generator now supports pinned local Iconify collection snapshots

The generator contract now accepts:

- `SourceSpec::SvgDirectory(...)`
- `SourceSpec::IconifyCollection(...)`

The current Iconify snapshot support resolves:

- base icons,
- aliases,
- inherited dimensions and view box inputs,
- `rotate`, `hFlip`, and `vFlip`,
- alias-loop detection,
- missing-parent detection,
- and explicit provenance that distinguishes `icons/<name>` from `aliases/<name>`.

Conclusion:

- the lane now has two local, deterministic source modes under the same reusable library surface,
- and the snapshot-based path does not require another output contract.

### 2) The public CLI surface now teaches snapshot import explicitly

The public command surface now includes:

- `fretboard icons import svg-dir --source <dir> --crate-name <name> --vendor-namespace <ns>`
- `fretboard icons import iconify-collection --source <file> --crate-name <name> --vendor-namespace <ns>`

Important behavior:

- both commands stay local-input-only,
- both generate the same repo-committable pack-crate shape,
- and both preserve the explicit app/bootstrap install seams already taught by first-party packs.

Conclusion:

- the external-developer producer story now covers the two accepted v1 local-input modes,
- without depending on repo-local Python scripts or a live network fetch contract.

### 3) Multi-color icon bodies survive the import path unchanged

The snapshot importer now has direct regression coverage for multi-color SVG bodies.

What this proves:

- the generator does not collapse Iconify bodies into a monochrome-only representation,
- embedded `<path ... fill=... />` content is preserved in the generated SVG asset bytes,
- and Fret's icon-pack boundary is compatible with colored vendor glyphs as long as the source SVG
  content already carries that color information.

Conclusion:

- the generator substrate is compatible with Iconify-style multicolor authoring at the asset level,
- and future collection-specific import crates do not need another runtime icon format to support
  colored icons.

### 4) Snapshot-based generation stays on the same compile-proof contract

The proof tests now:

- generate a real pack crate from a local Iconify collection JSON file,
- run `cargo check --features app-integration` against that generated crate,
- verify the generated `pack-provenance.json` records `iconify-collection`,
- and verify the generated icon inventory includes alias-derived outputs.

Conclusion:

- `IIPG-050` is satisfied without widening the producer contract,
- and the canonical generator proof gate now covers snapshot-based generation without network
  access, which satisfies `IIPG-052`.

## Decision from this slice

Treat the following M4 follow-ons as landed:

1. local Iconify collection snapshot input support in the reusable generator library,
2. explicit public CLI exposure for that input mode,
3. and snapshot-based regression coverage on the canonical generator proof gate.

The lane remains active only for the still-open follow-on:

- explicit public semantic alias configuration (`IIPG-051`).
