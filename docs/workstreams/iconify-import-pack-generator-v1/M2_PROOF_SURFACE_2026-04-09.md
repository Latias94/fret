# M2 Proof Surface — 2026-04-09

Status: landed proof slice

Status note (2026-04-09): this remains the first SVG-directory proof read for the lane. The
current follow-on status for user-facing docs lives in
`docs/workstreams/iconify-import-pack-generator-v1/MILESTONES.md`; the landed local Iconify
collection snapshot expansion lives in
`docs/workstreams/iconify-import-pack-generator-v1/M4_ICONIFY_COLLECTION_PROOF_2026-04-09.md`.

Related:

- `docs/workstreams/iconify-import-pack-generator-v1/DESIGN.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/TODO.md`
- `docs/workstreams/iconify-import-pack-generator-v1/MILESTONES.md`
- `docs/workstreams/iconify-import-pack-generator-v1/EVIDENCE_AND_GATES.md`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/svg_dir.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`

## Purpose

This note records the first landed M2 proof for `iconify-import-pack-generator-v1`.

The proof goal was intentionally narrow:

- do not solve remote Iconify fetch,
- do not solve every future pack source,
- and do not bless repo-local Python scripts as the public producer surface.

Instead, prove that the frozen M1 contract can already land as a real implementation slice:

- one reusable generator library,
- one thin public CLI entrypoint,
- one local SVG directory source,
- and one generated pack crate that compiles against the shipped Fret pack contract without manual
  cleanup.

## Commands used

Generator/library + CLI proof:

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo check -p fretboard --quiet`

Existing shipped icon-pack contract regression checks:

- `cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix`
- `cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"`

## Result summary

### 1) The first reusable implementation is library-first, not script-first

The proof now has a dedicated reusable library crate:

- `crates/fret-icons-generator`

Current implemented source support:

- local SVG directory import

Current generated output:

- standalone `Cargo.toml`
- `README.md`
- `pack-provenance.json`
- `icon-list.txt`
- `src/generated_ids.rs`
- `src/lib.rs`
- `src/app.rs`
- `src/advanced.rs`
- vendored `assets/icons/*.svg`

Conclusion:

- the producer surface is no longer “partial generated files plus handwritten glue” for the proof
  path,
- and the reusable logic is not trapped inside `fretboard`.

### 2) The public command surface is now explicit

The public CLI entrypoint is now:

- `fretboard icons import svg-dir --source <dir> --crate-name <name> --vendor-namespace <ns>`

Important behavior:

- public mode emits versioned dependencies for standalone/out-of-tree use,
- repo-mode exists only as an internal proof/test harness so generated packs can compile against
  local path dependencies under `local/`,
- and the generated pack crate no longer inherits hidden workspace-only assumptions such as
  `workspace.dependencies`.

Conclusion:

- the external-developer story is now explicit in the command surface instead of being implied by
  repo scripts.

### 3) The generated crate fits the shipped Fret pack contract without manual cleanup

The proof test in `crates/fretboard/src/icons/mod.rs` now:

- creates a repo-local SVG inventory,
- runs the repo-mode generator path,
- emits a real standalone pack crate,
- and runs `cargo check --features app-integration` against that generated crate.

What this proves:

- `PACK_METADATA` is emitted with `IconPackImportModel::Generated`,
- `PACK` / `VENDOR_PACK` are emitted,
- `app::install(...)` and `advanced::install_with_ui_services(...)` are emitted,
- vendored SVG assets and generated ids are wired through the same registration shape taught by the
  first-party packs,
- and the proof does not require hand-editing the generated crate before it compiles.

### 4) Provenance is explicit and reviewable

The generated pack now emits:

- `README.md` for human-readable integration/provenance guidance
- `pack-provenance.json` for machine-readable source inventory + alias policy

Current provenance details include:

- source kind
- source label
- pack metadata
- imported icon inventory
- semantic alias configuration

Conclusion:

- `IconPackImportModel::Generated` is sufficient at the runtime contract layer,
- while finer import/source detail stays in emitted provenance artifacts instead of forcing a new
  runtime enum expansion.

## Decision from this proof

Treat M2 as satisfied for the v1 proof target:

- one proof source exists,
- the output is a real pack crate,
- the app/bootstrap install contract is exercised by compilation,
- and the public third-party story is no longer monorepo-only.

## What remains after this proof

The lane stayed active after this proof because:

1. improve user-facing docs beyond CLI help and lane notes,
2. decide whether to promote the current proof command into broader cookbook/docs guidance,
3. add follow-on source support such as Iconify collection snapshots without reopening the producer
   contract,
4. decide whether/when to expose semantic alias config on the public CLI.
