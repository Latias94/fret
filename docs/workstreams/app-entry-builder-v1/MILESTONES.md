# App Entry Builder v1 (Milestones)

## Status summary

- `M0` Design convergence: **Completed**
- `M1` Builder implementation: **Completed**
- `M2` Onboarding switch: **Completed**
- `M3` Extension-seam polish: **In progress**
- `M4` Optional closure entry: **Deferred / undecided**

## M0 ? Design convergence

**Status:** Completed

**What closed**

- Ownership settled on `ecosystem/fret`.
- Primary naming settled on `fret::App`.
- Ergonomic aliases settled on `FretApp` and `AppBuilder`.
- The recommended mental model is now builder-first, and the old top-level shorthand helpers are gone from `fret`.

**Evidence**

- `docs/workstreams/app-entry-builder-v1/DESIGN.md`
- `ecosystem/fret/src/lib.rs`

## M1 ? Builder implementation

**Status:** Completed

**What shipped**

- Builder-chain entry type implemented in `fret`.
- UI entry and view entry both supported.
- Default main-window fallback preserved.
- Builder remains backed by existing `UiAppDriver` / bootstrap wiring.

**Evidence**

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`

## M2 ? Onboarding switch

**Status:** Completed

**What shipped**

- Templates use the builder chain.
- Golden-path docs use the builder chain.
- Representative examples now use the builder chain instead of the older helper-first story.
- `ecosystem/fret/README.md` teaches the builder chain as the first recommendation.

**Evidence**

- `apps/fretboard/src/scaffold/templates.rs`
- `docs/examples/todo-app-golden-path.md`
- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret/README.md`

## M3 ? Extension-seam polish

**Status:** In progress

**What shipped already**

- `ui_with_hooks(...)` keeps driver configuration on the builder path.
- `view_with_hooks::<V>(...)` does the same for the view runtime path.
- Advanced users can keep using `UiAppBuilder` and `UiAppDriver` seams without dropping down early.
- A focused guardrail now keeps the crate-root story builder-only and locks README onboarding to the
  same entry model.

**What remains**

- Keep compile/doc regression coverage current if more builder conveniences land.
- Keep docs/examples consistent about when to stay on `fret` versus when to drop down.
- Decide whether more builder conveniences should become first-class.

**Evidence**

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/README.md`
- `tools/gate_fret_builder_only_surface.py`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md`

## M4 ? Optional closure entry

**Status:** Deferred / undecided

This remains intentionally unresolved. The default Fret posture is still function-pointer based.
Any closure entry, if it ever exists, must be explicit about tradeoffs and must not weaken the
current hotpatch-friendly default.
