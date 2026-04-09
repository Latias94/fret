# Iconify Import-Pack Generator v1 — Evidence and Gates

Status: Active
Last updated: 2026-04-09

## Smallest current repro

Use this sequence before changing code:

```bash
python3 tools/check_icons_generation.py --pack all
cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix
cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

What this proves:

- the current first-party codegen/import pipeline is internally consistent,
- the shipped pack contract is real in code rather than only in docs,
- bootstrap still exposes the explicit contract-aware pack registration seam,
- and the app-facing docs/tests still teach the explicit install story that generated packs must
  target.

Environment note at lane open on 2026-04-09:

- `python3 tools/check_icons_generation.py --pack all` is the correct canonical gate for generated
  vendor-pack consistency,
- but it currently requires the local upstream source trees under `third_party/lucide` and
  `third_party/radix-icons`,
- and this workspace is currently missing at least the Lucide submodule checkout, so the gate is
  environment-blocked here rather than contract-blocked.

## Evidence at lane open

- `BASELINE_AUDIT_2026-04-09.md` records the current codegen/tooling substrate and generated-pack
  shape.
- `M1_CONTRACT_FREEZE_2026-04-09.md` records the accepted producer contract for v1:
  - local pinned inputs only,
  - full pack-crate output,
  - explicit alias policy,
  - `Generated` provenance for generator-produced packs,
  - and a future CLI/library public surface rather than a `tools/`-only answer.
- `icon-system-extension-v1` is now closed on the v1 icon contract and pack metadata/install seam.
- The repo already has shared codegen substrate in:
  - `tools/gen_icons.py`
  - `tools/icon_codegen.py`
- The current import pipeline is still first-party/vendor-specific:
  - Lucide
  - Radix
- The current docs already teach the target generated-pack shape:
  - `PACK_METADATA`
  - `PACK` / `VENDOR_PACK`
  - explicit app/bootstrap installation seams
- An optional local reference exists in `repo-ref/dioxus-iconify/README.md`, but it is local-state
  only and not a dependency or normative API source.

## Gate set

### Generation consistency

```bash
python3 tools/check_icons_generation.py --pack all
```

### Pack contract

```bash
cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix
```

### Bootstrap contract

```bash
cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"
```

### App/doc surface

```bash
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

## Evidence anchors

- `docs/workstreams/iconify-import-pack-generator-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
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
