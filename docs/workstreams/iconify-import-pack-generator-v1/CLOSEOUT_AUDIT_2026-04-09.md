# Closeout Audit — 2026-04-09

Status: closed closeout record

Related:

- `docs/workstreams/iconify-import-pack-generator-v1/DESIGN.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M4_ICONIFY_COLLECTION_PROOF_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/TODO.md`
- `docs/workstreams/iconify-import-pack-generator-v1/MILESTONES.md`
- `docs/workstreams/iconify-import-pack-generator-v1/EVIDENCE_AND_GATES.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/semantic_aliases.rs`
- `crates/fretboard/src/icons/mod.rs`

## Verdict

This lane is now closed.

It successfully landed the v1 producer contract that was intentionally left open by
`icon-system-extension-v1`:

- a reusable generator library crate,
- a thin public CLI entrypoint,
- deterministic pack-crate output that matches the shipped Fret icon-pack contract,
- local SVG-directory and local Iconify collection snapshot inputs,
- explicit public semantic `ui.*` alias configuration,
- and regression gates that prove the generated crate compiles against the current app/bootstrap
  installation seams.

## What shipped

### 1) Reusable generator substrate

- `crates/fret-icons-generator` now owns the reusable producer surface.
- The generator emits a complete pack crate rather than partial artifacts only.
- Generated output keeps provenance explicit via `README.md` and `pack-provenance.json`.

### 2) Public CLI entrypoints

`fretboard` now exposes:

- `fretboard icons import svg-dir ...`
- `fretboard icons import iconify-collection ...`

Both stay local-input-only and generate repo-committable crates for out-of-tree users.

### 3) Explicit semantic alias policy

Semantic alias policy is now public, explicit, and versioned:

- CLI users pass `--semantic-aliases <file>`
- the file is a versioned JSON contract,
- only explicit `ui.*` aliases are accepted,
- and no vendor-to-semantic mapping is guessed by the generator.

This preserves the mechanism-vs-policy boundary from ADR 0065 and the lane's M1 freeze.

### 4) Closure of the original open questions

The lane's original unresolved questions are now closed:

1. supported v1 source boundary: closed on local SVG inventory + local Iconify snapshot,
2. output boundary: closed on full pack-crate emission,
3. provenance classification: closed on `IconPackImportModel::Generated`,
4. reusable surface ownership: closed on library + thin CLI,
5. semantic alias exposure: closed on explicit versioned config only.

## Gates that now define the shipped surface

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo check -p fretboard --quiet`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`
- `cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix`
- `cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"`

## Follow-on policy

Do not reopen this lane for:

- runtime Iconify fetch,
- automatic semantic alias inference,
- or generic pack-policy expansion that exceeds the frozen v1 producer contract.

If future work is needed, open a narrower follow-on such as:

- remote/pinned acquisition as a separate pre-step,
- a broader pack-level config document beyond semantic aliases,
- or collection-specific import ergonomics that do not change the shipped v1 pack contract.
