# Baseline Audit — 2026-04-09

This audit records the starting fact pattern for `iconify-import-pack-generator-v1`.

Goal:

- confirm what part of the current icon import/codegen pipeline is already generic,
- confirm what part of the current generated-pack shape is still handwritten,
- and freeze the non-goals so this follow-on does not reopen the already-closed icon contract lane.

## Audited evidence

Lane docs:

- `docs/workstreams/iconify-import-pack-generator-v1/DESIGN.md`
- `docs/workstreams/iconify-import-pack-generator-v1/TODO.md`
- `docs/workstreams/iconify-import-pack-generator-v1/MILESTONES.md`
- `docs/workstreams/iconify-import-pack-generator-v1/EVIDENCE_AND_GATES.md`

Predecessor closeout:

- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`

Current tooling:

- `tools/gen_icons.py`
- `tools/gen_lucide.py`
- `tools/gen_radix.py`
- `tools/icon_codegen.py`
- `tools/sync_icons.py`
- `tools/verify_icons.py`
- `tools/check_icons_generation.py`

Current first-party packs:

- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-icons-lucide/{README.md,icon-list.txt,src/lib.rs,src/app.rs,src/generated_ids.rs}`
- `ecosystem/fret-icons-radix/{README.md,icon-list.txt,src/lib.rs,src/app.rs,src/generated_ids.rs}`
- optional local reference: `repo-ref/dioxus-iconify/README.md`

## Findings

### 1. The current tooling is only partially generic

The repo already has a useful shared substrate:

- `tools/icon_codegen.py` owns generic normalization, collision detection, `icon-list.txt`
  emission, and `generated_ids.rs` emission.
- `tools/gen_icons.py` is a thin multi-pack entrypoint.

But the broader toolchain is still explicitly first-party/vendor-specific:

- `tools/gen_lucide.py` and `tools/gen_radix.py` hardcode source layout, namespace, pack name,
  and output paths for those two packs only.
- `tools/sync_icons.py` only understands `lucide` / `radix` / `all` and assumes the current
  `ecosystem/fret-icons-{pack}` directory layout.
- `tools/verify_icons.py` only scans for `lucide.*` and `radix.*` vendor ids.
- `tools/check_icons_generation.py` hardcodes the generated-file set and upstream source-tree
  locations for Lucide/Radix.

Conclusion:

- the repo already has enough substrate to avoid starting from zero,
- but there is not yet a general input/config contract for "generate me a new third-party pack".

### 2. The current first-party packs are only partially generated

The generated or source-derived parts today are:

- `icon-list.txt`
- `src/generated_ids.rs`
- vendored `assets/icons/*.svg`

The contract-bearing crate surface is still handwritten per pack:

- `src/lib.rs`
- `src/app.rs`
- `src/advanced.rs`
- README guidance
- semantic alias mapping policy

That handwritten surface is not accidental noise.
It currently carries the real Fret pack contract:

- `PACK_METADATA`
- `VENDOR_PACK`
- optional `UI_SEMANTIC_ALIAS_PACK`
- `PACK`
- `register_vendor_icons(...)`
- optional semantic alias registration
- explicit `app::install(...)`
- optional `advanced::install_with_ui_services(...)`

Conclusion:

- the current generator substrate does not yet generate a complete Fret pack crate,
- and one of the first M1 decisions must be whether the v1 generator should emit this full crate
  shape or continue to stop at assets + ids while leaving glue code manual.

### 3. The shipped pack contract is now explicit enough to target directly

`fret-icons` already exposes the stable nouns a generated pack should target:

- `IconPackImportModel`
- `IconPackMetadata`
- `IconPackRegistration`
- `InstalledIconPacks`

The first-party packs already prove the intended output shape:

- provenance lives in `PACK_METADATA`,
- registration values are data-first constants,
- app/bootstrap integration stays explicit,
- and installed-pack metadata is recorded separately from raw registry bytes.

Conclusion:

- this lane does not need to invent another output surface,
- it should target the pack contract that already shipped.

### 4. A dioxus-iconify-style workflow is relevant, but only as workflow reference

The optional local `repo-ref/dioxus-iconify/README.md` reference is useful because it validates
several directional choices:

- build-time generation,
- repo-committable output,
- local SVG support,
- and a generator-first user workflow.

However, it is not the source of truth for Fret because:

- `repo-ref/` is local state,
- its runtime component API is framework-specific,
- and Fret already has a separate semantics-first icon/runtime contract.

Conclusion:

- borrow the workflow posture,
- do not borrow the runtime API shape.

### 5. The current canonical generation gate is environment-sensitive

`python3 tools/check_icons_generation.py --pack all` is the right gate for the existing
first-party generator pipeline, but it currently depends on local upstream source trees:

- `third_party/lucide/icons`
- `third_party/radix-icons/packages/radix-icons/icons`

In this workspace, the Lucide source tree is missing, so the gate is currently blocked by local
environment state rather than by a contract failure.

Conclusion:

- the lane should treat upstream source acquisition/bootstrap as an explicit precondition,
- not as an unexplained generator failure mode.

## M0 verdict

The M0 scope/evidence freeze is now specific enough to continue:

- the predecessor contract is explicit,
- the current generator substrate and its limits are explicit,
- the current generated-pack shape is explicit,
- and the non-goals are explicit.

From this point, the next real work is M1:

- freeze the v1 input boundary,
- freeze the v1 output boundary,
- and decide whether the generator owns the full crate surface or only a narrower generated core.
