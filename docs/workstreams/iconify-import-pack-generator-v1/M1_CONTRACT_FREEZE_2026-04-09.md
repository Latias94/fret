# M1 Contract Freeze — 2026-04-09

Status: accepted v1 decision

Related:

- `docs/workstreams/iconify-import-pack-generator-v1/DESIGN.md`
- `docs/workstreams/iconify-import-pack-generator-v1/TODO.md`
- `docs/workstreams/iconify-import-pack-generator-v1/MILESTONES.md`
- `docs/workstreams/iconify-import-pack-generator-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/iconify-import-pack-generator-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `tools/gen_icons.py`
- `tools/gen_lucide.py`
- `tools/gen_radix.py`
- `tools/icon_codegen.py`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-radix/src/lib.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `repo-ref/dioxus-iconify/README.md`

## Purpose

This note closes the M1 question set for `iconify-import-pack-generator-v1`.

The goal is to freeze the smallest correct producer contract before implementation drift starts:

- which inputs count as supported source material,
- whether generation stops at partial artifacts or emits a real Fret pack crate,
- how semantic alias policy is kept explicit instead of guessed,
- how generated packs classify provenance,
- and whether the reusable surface is only repo-local tooling or a real external-developer entrypoint.

## Frozen decisions

### 1) The v1 generator consumes explicit local sources only; network fetch is not part of the contract

Decision:

- the supported v1 source boundary is:
  - local Iconify-style collection snapshot files,
  - local SVG files and directories,
  - and explicit local pack configuration for namespace, pack id, alias policy, and provenance;
- the generator must not treat remote Iconify HTTP fetch as part of its stable contract;
- and any future remote acquisition convenience should be a separate pre-step or follow-on lane,
  not hidden inside the generator itself.

Why:

- Fret's icon-pack story is intentionally build-time, deterministic, and repo-committable;
- the existing first-party gates already show that source acquisition is an environmental
  precondition, not something the pack contract should hide;
- and provenance/licensing become harder to audit when generation silently depends on live network
  state.

Operational consequence:

- “generate a pack” means “consume pinned local inputs and emit reviewable crate output”;
- “fetch from Iconify” is not the same contract and must stay visibly separate if it exists later.

### 2) The v1 output boundary is a complete Fret icon-pack crate, not only `assets + ids`

Decision:

- the generator must target the full shipped Fret pack contract,
- it must emit a normal Rust crate that is directly consumable by the existing app/bootstrap
  surfaces,
- and it must not leave the developer to hand-author the contract-bearing glue after generation.

Minimum crate-level outcome:

- vendored SVG assets under a normal pack-owned asset layout,
- generated vendor ids/constants,
- explicit `PACK_METADATA`,
- explicit `PACK` / `VENDOR_PACK`,
- explicit registry entrypoints such as `register_vendor_icons(...)` / `register_icons(...)`,
- explicit app-facing install surface (`app::install(...)`),
- explicit advanced/bootstrap-facing install surface when the pack opts into app integration,
- and reviewable README/provenance output explaining what was imported.

Important boundary:

- the stable contract is the crate-level exported surface and deterministic emitted artifacts,
  not every private helper filename inside `src/`;
- implementation is free to choose the exact internal generated module split so long as developers
  do not need manual cleanup to reach the shipped Fret pack shape.

Why:

- the current missing surface is the producer contract, not another runtime or bootstrap contract;
- stopping at `icon-list.txt` plus `generated_ids.rs` would preserve the current gap for
  third-party developers;
- and the docs already teach custom packs as real crates consumed through
  `my_icons::app::install` or `BootstrapBuilder::register_icon_pack_contract(my_icons::PACK)`.

### 3) Semantic alias policy is opt-in and explicit; the generator must not infer `ui.*` aliases

Decision:

- semantic alias registration is optional generator input,
- any emitted semantic alias module must come only from explicit configuration,
- and the generator must not guess `ui.*` mappings from vendor icon names, popularity, or another
  framework's defaults.

Why:

- semantic alias choice is policy, not mechanism;
- first-party Lucide/Radix crates already prove that alias mapping is curated and pack-specific;
- and auto-inferring semantic aliases would blur the exact layer boundary that `fret-icons` and
  `fret-ui` are trying to protect.

Operational consequence:

- v1 may scaffold or emit alias registration code,
- but only when the developer provided the mapping deliberately.

### 4) `IconPackImportModel::Generated` is the correct v1 classification for generator-produced packs

Decision:

- a pack crate produced by this generator should classify itself as
  `IconPackImportModel::Generated`,
- `IconPackImportModel::Vendored` remains the label for crates whose maintained source of truth is
  vendored upstream trees/submodules,
- `IconPackImportModel::Manual` remains the escape hatch for hand-authored packs,
- and no new import-model enum variant is justified in v1.

Important nuance:

- `Generated` describes how the pack crate was produced,
- not every upstream licensing/source detail of the icons inside it;
- fine-grained provenance belongs in generated machine-readable metadata plus human-readable README
  output, not in another top-level enum value.

Why:

- `fret-icons` already has enough provenance vocabulary for this producer lane;
- adding another enum now would create contract churn before there is a real second consumer
  demanding it;
- and generated packs can still carry richer source details without widening the runtime-facing
  metadata surface.

### 5) The stable reusable surface should be a public CLI backed by a reusable library crate, not repo-local `tools/` scripts

Decision:

- repo-local Python scripts are valid baseline evidence and migration substrate,
- but they are not the correct final third-party developer surface;
- the v1 reusable surface should be:
  - a thin public CLI as the user-facing entrypoint,
  - backed by a reusable library crate that owns config parsing, planning, normalization, and
    deterministic emission;
- and the generated pack must have no runtime dependency on that generator crate.

Why:

- this lane exists specifically to make out-of-tree third-party pack generation real;
- a repo-local `tools/`-only answer would keep the workflow monorepo-bound;
- and a library + CLI split avoids later rewrites when tests, fixtures, or IDE/tooling integrations
  need the same planner without shelling out to ad-hoc scripts.

Clarification:

- this does not force immediate publication before proof exists;
- it freezes the ownership direction so M2/M3 implementation does not accidentally bless the
  temporary Python scripts as the long-term public contract.

## Rejected alternatives

### Runtime Iconify fetch inside the stable generator command

Rejected because:

- it hides nondeterministic network state inside what should be a reproducible build-time import
  step,
- it weakens provenance review,
- and it conflates source acquisition with pack emission.

### Keep generating only partial artifacts and let developers hand-write the rest

Rejected because:

- it preserves the exact third-party producer gap this lane exists to close,
- it keeps Fret's real pack contract implicit,
- and it invites drift between docs, first-party packs, and generated out-of-tree packs.

### Auto-map upstream icons onto semantic `ui.*` ids

Rejected because:

- semantic alias policy is curated and install-order-sensitive,
- different products may want different semantic ownership,
- and the generator should not decide design-system taste on behalf of pack authors.

### Bless repo-local `tools/` as the final external-developer answer

Rejected because:

- the whole point of this lane is an out-of-tree story,
- repo-local scripts do not define a stable public contract,
- and a later CLI/library extraction would become avoidable churn.

## Immediate consequences

From this point forward:

1. treat local pinned source material as the only supported v1 generator input;
2. treat “full pack crate emission” as the required producer outcome;
3. treat semantic aliases as explicit config-driven policy, never inferred policy;
4. treat `IconPackImportModel::Generated` as sufficient for generator-produced packs;
5. treat repo-local scripts as transitional substrate, not as the final reusable surface.

## What M2 now needs to prove

The next implementation/proof slice should verify:

- one generator path can start from local Iconify-style data or local SVG inventory,
- the emitted output lands as a real Fret pack crate with no manual cleanup,
- the emitted crate plugs into `my_icons::app::install` and
  `BootstrapBuilder::register_icon_pack_contract(my_icons::PACK)`,
- provenance output is reviewable and durable,
- and the proof does not rely on monorepo-only assumptions.
