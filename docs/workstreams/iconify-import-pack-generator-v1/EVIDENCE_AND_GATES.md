# Iconify Import-Pack Generator v1 — Evidence and Gates

Status: Active
Last updated: 2026-04-09

## Smallest current repro

Use this sequence before changing code:

```bash
cargo nextest run -p fret-icons-generator -p fretboard
cargo check -p fretboard --quiet
cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix
cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"
```

What this proves:

- the current generator library + public CLI proof surface is internally consistent across both
  accepted local-input modes,
- the generated-pack proof still compiles against the shipped app integration contract,
- the public CLI still exposes the external-developer producer story,
- the shipped pack contract is real in code rather than only in docs,
- bootstrap still exposes the explicit contract-aware pack registration seam,
- and the earlier first-party icon-pack contract still stays intact.

Environment note for the legacy first-party vendor-generation gate:

- `python3 tools/check_icons_generation.py --pack all` is the correct canonical gate for generated
  Lucide/Radix vendor-pack consistency,
- but it currently requires the local upstream source trees under `third_party/lucide` and
  `third_party/radix-icons`,
- so it remains relevant when changing the old first-party Python import pipeline,
- but it is no longer the only meaningful gate for this lane now that the M2 proof surface lives
  in `crates/fret-icons-generator` + `fretboard`.

## Current evidence set

- `BASELINE_AUDIT_2026-04-09.md` records the current codegen/tooling substrate and generated-pack
  shape.
- `M1_CONTRACT_FREEZE_2026-04-09.md` records the accepted producer contract for v1:
  - local pinned inputs only,
  - full pack-crate output,
  - explicit alias policy,
  - `Generated` provenance for generator-produced packs,
  - and a future CLI/library public surface rather than a `tools/`-only answer.
- `M2_PROOF_SURFACE_2026-04-09.md` records the landed proof slice:
  - `crates/fret-icons-generator` as the reusable library,
  - `fretboard icons import svg-dir` as the thin public entrypoint,
  - repo-local generated-pack compilation proof,
  - emitted `README.md` + `pack-provenance.json`.
- `M4_ICONIFY_COLLECTION_PROOF_2026-04-09.md` records the landed source-expansion follow-on:
  - `crates/fret-icons-generator/src/iconify.rs` as the local Iconify snapshot importer,
  - `fretboard icons import iconify-collection` as the second public local-input entrypoint,
  - multicolor SVG-body preservation coverage,
  - and snapshot-based generated-pack compilation proof without network fetch.
- User-facing docs now teach both current public generator entrypoints in:
  - `docs/crate-usage-guide.md`
  - `docs/examples/todo-app-golden-path.md`
  - guarded by `ecosystem/fret/src/lib.rs`
- `icon-system-extension-v1` is now closed on the v1 icon contract and pack metadata/install seam.
- The repo now has two relevant producer/tooling surfaces:
  - old first-party vendor-specific Python import/generation scripts,
  - new reusable Rust generator + public CLI proof surface.
- The current docs already teach the target generated-pack shape:
  - `PACK_METADATA`
  - `PACK` / `VENDOR_PACK`
  - explicit app/bootstrap installation seams
- An optional local reference exists in `repo-ref/dioxus-iconify/README.md`, but it is local-state
  only and not a dependency or normative API source.

## Gate set

### Generator proof

```bash
cargo nextest run -p fret-icons-generator -p fretboard
```

This gate now covers:

- SVG-directory generation,
- Iconify collection snapshot generation,
- multicolor Iconify body preservation,
- and repo-local generated-pack compile proof for both source kinds.

### Public CLI surface

```bash
cargo check -p fretboard --quiet
```

### Pack contract

```bash
cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix
```

### Bootstrap contract

```bash
cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"
```

### Legacy first-party vendor generation consistency

```bash
python3 tools/check_icons_generation.py --pack all
```

## Evidence anchors

- `docs/workstreams/iconify-import-pack-generator-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M4_ICONIFY_COLLECTION_PROOF_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `ecosystem/fret/src/lib.rs`
- `crates/fret-icons-generator/Cargo.toml`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/fs.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/naming.rs`
- `crates/fret-icons-generator/src/svg_dir.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `tools/gen_icons.py`
- `tools/icon_codegen.py`
- `tools/sync_icons.py`
- `tools/verify_icons.py`
- `tools/check_icons_generation.py`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-lucide/README.md`
- `ecosystem/fret-icons-radix/src/lib.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `ecosystem/fret-icons-radix/README.md`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret/src/lib.rs`

## Reference posture

- Optional local reference:
  - `repo-ref/dioxus-iconify/README.md`
- Read it for build-time generator workflow ideas only.
- Do not treat `repo-ref/` contents as dependencies or as the source of truth for Fret's public
  API shape.
