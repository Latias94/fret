# Fret Launch + App Surface (Fearless Refactor v1) — Surface Audit

Status: Post-example migration audit

Scope:

- `crates/fret-launch`
- `ecosystem/fret`
- `ecosystem/fret-bootstrap`
- `crates/fret-framework::launch`

This note answers the practical question behind the workstream: after curating docs and migrating
representative examples to `FnDriver` implementation paths, is the current launch/app surface
reasonable for general-purpose applications while still preserving editor-grade customization?

## Verdict

Short answer: **yes, mostly**.

The current surface already has the right capability split:

- `fret` is now a credible desktop-first, general-purpose app-author surface.
- `fret-launch` still exposes enough seams for host-integrated / editor-grade applications.
- `fret-bootstrap` is the correct bridge for callers who want bootstrap defaults without dropping all
  the way down to raw launch wiring.

The main remaining debt is **surface curation**, not missing power.

Compared with GPUI/Zed-style expectations, Fret is no longer blocked on extensibility. The core
question is now how aggressively to shrink and classify public exports, not whether the framework
can support advanced apps.

## Evidence-backed assessment

### 1) `fret` is sufficient for common apps

For a typical desktop app author, the relevant mental model is already close to:

- app
- main window
- root UI/view
- optional app-level install hooks

Evidence:

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/README.md`

Why this is enough:

- `fret::App::new(...).window(...).ui(...)` is the recommended short path.
- `fret::App::new(...).window(...).ui_with_hooks(...)` keeps advanced driver hooks on that same
  builder path.
- The builder chain is now the only `fret` app-author entry story, which removes first-contact
  ambiguity at the crate root.
- `UiAppBuilder` still exposes real extension points without forcing app authors to start from
  `fret-launch`.

### 2) The advanced seams are still first-class on `fret`

The `fret` facade already exposes the advanced hooks that matter for non-trivial products:

- `App::{ui_with_hooks, view_with_hooks::<V>}`
- `configure(...)`
- `on_gpu_ready(...)`
- `install_custom_effects(...)`
- window create/created/before-close hooks
- engine-frame recording
- viewport input and global command hooks

Evidence:

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`

This means advanced users can stay on `fret` longer than before, and only drop to
`fret-bootstrap` / `fret-launch` when they truly need runner-owned behavior.

### 3) `fret-launch` is strong enough for editor-grade integration

`fret-launch` still carries the seams that matter for serious host integration:

- host-provided GPU init (`WgpuInit`)
- per-window create specs
- event / command / render contexts
- viewport input
- docking ops
- engine-frame hooks
- accessibility hooks
- imported viewport / external texture interop

Evidence:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/src/runner/common/fn_driver.rs`

This is enough to support the same class of outcomes expected from editor-like shells, even though
the shape differs from GPUI.

### 4) The migration from trait-driver examples materially reduced ambiguity

Representative examples now route their runtime path through `FnDriver`, including:

- chart/plot demos
- form/table/date picker demos
- stress demos
- docking demos
- workspace shell
- node graph legacy demo

Evidence:

- `apps/fret-examples/src/`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/TODO.md`

This matters because examples define the real public story more strongly than prose docs do.

## GPUI comparison

### What is now comparable

Fret now supports the same broad layering outcome:

- simple app-author entry path
- advanced integration escape hatch
- editor-grade multi-window / docking / viewport customization

### What is intentionally different

Fret’s advanced runtime posture is still explicitly **function-pointer / hook based** (`FnDriver`),
not a direct GPUI-style closure runtime.

That difference is acceptable because it buys a few things Fret explicitly values:

- hotpatch-friendly function boundaries
- clearer separation between mechanism hooks and app/bootstrap defaults
- easier preservation of native/web runner wiring as an explicit layer

### Current conclusion

Fret no longer lacks extensibility relative to the intended target. What it still lacks is a tighter
curation story around which names are the stable/default ones.

## Recent closure

### `fret-framework::launch` is now a curated subset

`crates/fret-framework/src/lib.rs` now re-exports the core manual-assembly launch contract instead
of mirroring the full `fret_launch::*` surface.

Included surface (high level):

- `FnDriver` / `FnDriverHooks`
- `WinitAppDriver` plus the core runner contexts
- `WinitRunnerConfig`, `WgpuInit`, and window-spec types
- top-level app entry wiring (`WinitAppBuilder`, `run_app*`, wasm handle entrypoints)

Left on `fret-launch` directly:

- specialized media helpers
- imported viewport / external texture interop helpers
- other launch-root exports that are real but not part of the compact framework facade

Evidence:

- `crates/fret-framework/src/lib.rs`

Interpretation:

- manual assembly keeps a compact umbrella path,
- specialized integrations still have a direct escape hatch,
- the previous full-mirror hazard is now closed in this worktree.

### Specialized interop/media helpers now live under explicit modules

The `fret-launch` public surface now distinguishes between:

- core launch/builder/driver contracts kept at crate root,
- specialized interop helpers under `imported_viewport_target` / `native_external_import`,
- platform media helpers under `media`,
- shared-allocation interop under `shared_allocation`.

That is a meaningful curation improvement because advanced helpers remain public without pretending
to be part of the first-contact crate-root story.

Evidence:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/README.md`

## Remaining hazards

### H1) `WinitRunnerConfig` is stable but still over-broad

The launch config is still a single object that mixes:

- app/window defaults
- backend tuning
- streaming/media tuning
- platform/web specifics

Evidence:

- `crates/fret-launch/src/runner/common/config.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`

Interpretation:

- keep it stable for now,
- prefer helper-layer curation over a breaking shape split.

### H2) `WinitAppDriver` remains public as compatibility surface

This is now well documented, but it is still real public API.

Evidence:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/README.md`

Interpretation:

- acceptable short term,
- but future removal/de-emphasis still needs explicit sunset criteria.

### H3) Specialized launch modules still need classification discipline

Several specialized interop/media/render-target exports are valid, but they should stay
classification-driven and should not expand casually even now that they live under explicit
submodules.

Evidence:

- `crates/fret-launch/src/lib.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`

## Recommended next steps

1. **Keep `fret-framework::launch` curated**
   - Outcome: future additions require explicit facade-level justification; specialized helpers stay on `fret-launch`.
   - Gate: `cargo nextest run -p fret-framework --no-tests pass`, `python tools/check_layering.py`

2. **Lean into helper-layer config curation**
   - Outcome: keep `WinitRunnerConfig` stable, but expose more app-facing config helpers through
     `fret` / `fret-bootstrap` instead of teaching the full config object by default.
   - Gate: `cargo nextest run -p fret -p fret-bootstrap`

3. **Define a sunset bar for `WinitAppDriver`**
   - Outcome: write down which missing hook(s), if any, still block stronger de-emphasis.
   - Gate: docs update + representative example coverage remains green.

4. **Keep specialized launch modules classification-driven**
   - Outcome: future specialized additions land under explicit modules and must still be tagged as stable / specialized / transitional.
   - Gate: export inventory stays current.

## Final judgment

For the original question — “is the launcher/internal surface reasonable, and can users of the
public `fret` facade extend/customize enough like Zed/GPUI while still being suitable for general
apps?” — the current answer is:

- **General-purpose apps:** yes
- **Advanced/editor-grade customization:** yes
- **Main remaining problem:** curation / naming / export discipline, not capability

That is a good place to be before publication.

## Gates run for this audit phase

- `cargo fmt -p fret-examples`
- `cargo check -p fret-examples -p fret-demo-web --all-targets`
- `cargo nextest run -p fret-examples`
- `cargo fmt -p fret-framework`
- `cargo nextest run -p fret-framework --no-tests pass`
- `python tools/check_layering.py`

## Evidence anchors

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/src/runner/common/fn_driver.rs`
- `crates/fret-framework/src/lib.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/TODO.md`
