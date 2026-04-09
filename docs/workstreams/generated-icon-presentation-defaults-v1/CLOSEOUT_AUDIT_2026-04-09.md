# Closeout Audit — 2026-04-09

Status: closed closeout record

Related:

- `docs/workstreams/generated-icon-presentation-defaults-v1/DESIGN.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/TODO.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/MILESTONES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/presentation_defaults.rs`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/help.rs`
- `ecosystem/fret/src/lib.rs`

## Verdict

This lane is now closed.

It successfully landed the narrow follow-on that both the runtime icon lane and the acquisition
lane intentionally left open:

- explicit generated-pack presentation-default policy,
- a versioned public config contract,
- thin CLI exposure for both local SVG and local Iconify snapshot imports,
- explicit generated registration/provenance output,
- preserved runtime `Mask` vs `OriginalColors` behavior,
- and public docs/source-policy gates that teach the shipped surface.

## What shipped

### 1) Explicit presentation policy now belongs to generator/import time

The shipped v1 rule is explicit and reviewable:

- generator users pass a versioned `presentation-defaults.json`,
- `fret-icons-generator` validates and applies it before code generation,
- and runtime consumers continue to honor `IconPresentation` rather than discovering policy later.

This keeps policy in the correct layer and matches the lane's M1 freeze.

### 2) Generated output now preserves presentation in code and provenance

Generated packs no longer collapse every imported icon onto the default mask-mode posture.

Instead, generated output now makes presentation visible in:

- explicit `register_svg_bytes_with_presentation(...)` registrations,
- generated README guidance,
- and `pack-provenance.json` with both pack defaults and per-icon resolved render modes.

### 3) The runtime split stayed closed and is now proven reusable

This lane did not reopen runtime rendering.

The explicit authored-color path remains:

- registry carries `IconPresentation`,
- `icon_authored(...)` uses `SvgImage` for `OriginalColors`,
- and mask-mode icons stay on the themed `SvgIcon` posture.

### 4) The public teaching surface now matches the shipped contract

The public docs and source-policy gates now teach:

- `--presentation-defaults ./presentation-defaults.json`,
- the versioned config shape,
- the pack-level default plus per-icon overrides model,
- and the rule that `icon_name` must use the generated icon name.

This closes the remaining M3 doc gap for the lane.

## Gates that now define the shipped surface

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo nextest run -p fret-ui-kit --features icons -E 'package(fret-ui-kit) & (test(declarative::icon::tests::icon_authored_uses_svg_image_for_original_color_icons) | test(declarative::icon::tests::icon_authored_uses_svg_icon_for_mask_icons))'`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`

## Follow-on policy

Do not reopen this lane for:

- automatic presentation inference from Iconify `palette`,
- SVG heuristics as a silent default,
- or runtime-side presentation rediscovery.

If future work is needed, open a narrower follow-on such as:

1. helper tooling that derives or suggests `presentation-defaults.json` from acquisition
   provenance,
2. explicit SVG-analysis scaffolding that produces suggested config rather than silent policy,
3. or broader first-party pack curation policy outside the generated/imported-pack contract.

