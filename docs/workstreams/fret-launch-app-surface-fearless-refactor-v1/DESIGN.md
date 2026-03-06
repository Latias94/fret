# Fret Launch + App Surface (Fearless Refactor v1)

## Context

Fret already has the right **macro layering** for a long-lived UI framework:

- `crates/*` hold portable contracts, runtimes, backends, and runner glue.
- `ecosystem/*` holds policy-heavy defaults, recipes, and app-facing convenience.
- `apps/fretboard` owns dev workflows instead of pushing toolchain concerns into libraries.

That macro story is coherent with ADR 0109 and with the repo's mechanism-vs-policy split.

However, the **user-facing launch story** is still more complicated than it should be:

- `crates/fret-launch` intends to be a stable launch facade, but its root public surface still re-exports a large amount of runner plumbing.
- `crates/fret-framework` is a clean manual-assembly facade, but it currently mirrors `fret-launch` power more than it curates it.
- `ecosystem/fret` already provides a good desktop-first golden path, but advanced customization drops users into runner-centric concepts relatively early.

Compared with the reference posture in `repo-ref/zed/crates/gpui`, Fret is not missing power. It is missing a more strongly curated **surface contract**.

This workstream is documentation-first. It records a staged refactor plan before we move public API around.

## What is already working

### 1) The crate layering is directionally correct

- `fret-framework` is a small opt-in facade for manual assembly.
- `fret` is the batteries-included ecosystem surface.
- `fret-launch` is where runner/platform/render wiring belongs.

This is the right split for a framework that wants both app-author ergonomics and advanced host integration.

### 2) The advanced integration seams are strong

Today the launch stack already supports capabilities that matter for general-purpose and editor-grade apps:

- host-provided `WgpuContext` and factory-driven GPU initialization,
- event-loop customization,
- per-window create specs,
- window lifecycle interception,
- custom engine-frame recording,
- viewport input and docking hooks,
- dev-state / hotpatch-friendly `fn`-pointer hooks.

That means the core problem is **surface shape**, not missing capability.

### 3) The ecosystem golden path is viable

`ecosystem/fret` already wraps the bootstrap/driver layers into a usable builder path for desktop apps.

The gap is that the transition from "simple app author" to "advanced integrator" still exposes more runner detail than we likely want as a long-term contract.

## Problem statement

We want these three statements to be true at the same time:

1. A typical app author can stay on `fret` and build a general-purpose app without understanding runner internals.
2. An advanced integrator can still assemble custom launch behavior without forking the stack.
3. `fret-launch` can evolve internally without every internal module becoming a de facto public contract.

Today, (2) is largely true, but (1) and especially (3) are only partially true.

## Key hazards discovered in the audit

### H1) Public-surface mismatch inside `fret-launch`

`crates/fret-launch/README.md` says `runner/` is internal plumbing that may evolve, but `crates/fret-launch/src/lib.rs` publicly re-exports many runner types directly.

This creates contract drift:

- docs imply implementation freedom,
- exports imply stability obligations.

### H2) Dual advanced driver story

Both `WinitAppDriver` and `FnDriver` are public. The code already states the intended direction: once `FnDriver` covers all required hooks, `WinitAppDriver` should leave the public surface.

Until that cleanup lands, advanced users see two overlapping ways to do the same job.

### H3) `WinitRunnerConfig` mixes stable app knobs with backend-heavy detail

The current config surface carries:

- window defaults,
- accessibility toggles,
- renderer budgets,
- SVG budgets,
- streaming upload budgets,
- web canvas identity,
- experimental GPU conversion switches.

This is powerful, but it is not a great long-term **single stable config object** for app authors.

### H4) The golden path is desktop-first, but the naming can suggest broader symmetry than exists today

The high-level `fret::App` / `UiAppBuilder` story is excellent for native desktop work, but it is not yet a fully symmetric cross-platform authoring surface.

That is fine for now, as long as the docs are explicit and the contract surface does not over-promise.

### H5) The app-author mental model becomes runner-centric too early

GPUI's primary authoring story feels like:

- application,
- app context,
- window,
- root view.

Fret's advanced story quickly becomes:

- driver,
- runner config,
- engine frame,
- window creation hooks.

That is appropriate for framework integrators, but not ideal as the main public mental model.

## Goals

- Keep `fret` as the recommended app-author entry surface.
- Keep `fret-framework` as the manual-assembly facade.
- Keep `fret-launch` powerful enough for custom host integration.
- Narrow the stable `fret-launch` contract so internal runner refactors remain possible.
- Preserve the current hotpatch-friendly `fn`-pointer posture by default.
- Make it easier to explain the story in the same terms every time: **app path vs integration path**.

## Non-goals

- Rewriting the retained/declarative UI runtime.
- Moving policy behavior into `crates/fret-ui`.
- Redesigning docking semantics or the command system.
- Removing advanced host-integration seams such as provided GPU contexts or event-loop hooks.
- Forcing a GPUI-style closure-first runtime across the stack.

## Invariants (must remain true)

1. `crates/fret-ui` remains a mechanism/contract layer, not a policy/component layer.
2. `fret` remains an ecosystem-level convenience surface.
3. `fret-framework` remains suitable for manual/advanced assembly.
4. `fret-launch` remains the place for native/web launch glue and host integration seams.
5. `crates/*` must not depend on `ecosystem/*`.
6. Host-provided GPU / event-loop integration remains possible.

## Proposed target shape

### A) Clarify the three user-facing surfaces

#### `ecosystem/fret`

This is the **recommended app-author path**.

It should optimize for:

- minimal ceremony,
- stable naming,
- practical defaults,
- clear extension hooks,
- not forcing users to reason about runner internals.

#### `crates/fret-framework`

This is the **manual assembly path**.

It should remain a thin, feature-gated facade over framework crates and launch crates, without pretending to be the easiest beginner surface.

#### `crates/fret-launch`

This is the **advanced launch/integration path**.

It should expose the stable seams needed by integrators, but it should stop advertising whole internal namespaces as if they were long-term API commitments.

### B) Narrow `fret-launch` to a curated stable surface

The intended stable surface should be small and explicit. Directionally, it should include only things such as:

- `RunnerError`,
- `FnDriver`,
- `FnDriverHooks`,
- core runner contexts,
- `WinitRunnerConfig`,
- `WgpuInit`,
- `WindowCreateSpec`,
- `WinitAppBuilder`,
- top-level `run_app*` entry points,
- a small set of host-integration helper types that are intentionally supported.

Everything else should be one of:

- internal-only,
- `#[doc(hidden)]` transitional,
- or a later explicit contract decision.

### C) Make `FnDriver` the single recommended advanced driver surface

`FnDriver` already matches the repo's hotpatch-friendly posture better than the trait-based path.

Planned direction:

1. audit all remaining use cases that still need `WinitAppDriver`,
2. fill any missing `FnDriverHooks` gaps,
3. mark `WinitAppDriver` as compatibility-only,
4. move toward a single advanced driver story.

This reduces API overlap and makes documentation simpler.

### D) Split public app-facing config from backend-heavy tuning

`WinitRunnerConfig` is currently doing too much. We should move toward a shape where:

- common app-facing knobs stay obvious,
- backend/perf/cache tuning stays available,
- but the top-level public config object is easier to reason about.

This does **not** require an immediate breaking redesign. A staged plan is acceptable:

1. document sub-groups conceptually,
2. add builder helpers or nested config sections,
3. de-emphasize advanced fields in beginner-facing docs,
4. eventually decide whether the public config type should be split.

### E) Keep host integration first-class

The following are not accidental complexity; they are framework strengths and should remain supported:

- supplied GPU context,
- GPU factory callback,
- event-loop customization,
- window creation interception,
- engine-frame customization,
- foreign/embedded viewport interop.

The refactor should protect those seams while curating how they are presented.

### F) Keep the `fret` story coordinated with `app-entry-builder-v1`

This workstream should not duplicate the app-entry builder work.

Relationship between the two workstreams:

- `app-entry-builder-v1` is about **ergonomic author-facing entry composition**.
- this workstream is about **stable launch substrate and public-surface boundaries**.

The builder workstream should sit on top of a cleaner launch/app surface, not compensate for unclear substrate boundaries forever.

## Staging strategy

### Stage 0 - Documentation alignment

- land this folder,
- align docs with the intended three-surface story,
- explicitly record the current hazards and target direction.

### Stage 1 - Surface inventory and contract classification

- classify every `fret-launch` root export as:
  - stable public,
  - transitional public,
  - internal plumbing.
- identify what `fret` needs to re-export or wrap versus what should stay integration-only.

### Stage 2 - Single advanced driver path

- finish any `FnDriverHooks` gaps,
- reduce dependence on public `WinitAppDriver`,
- update advanced docs/examples accordingly.

### Stage 3 - Config curation

- define app-facing vs backend-heavy config categories,
- update builder/facade docs to steer most users to the right layer,
- keep deep tuning available for integrators.

### Stage 4 - Cross-surface doc polish

- ensure `fret`, `fret-framework`, and `fret-launch` docs describe distinct roles,
- ensure GPUI comparison is used as a mental-model reference rather than an implementation mandate.

## Definition of done

This workstream is complete when:

1. `fret-launch` no longer presents large internal runner namespaces as accidental stable API.
2. advanced driver documentation clearly recommends one path.
3. `fret` docs clearly describe the app-author path and the escape hatch to lower layers.
4. host integration capabilities remain available without private patching.
5. the public story is easy to explain in one sentence per crate:
   - `fret`: build apps,
   - `fret-framework`: assemble the framework manually,
   - `fret-launch`: integrate or customize launch/runtime wiring.

## Evidence anchors

- Launch facade exports: `crates/fret-launch/src/lib.rs`
- Launch crate ownership note: `crates/fret-launch/README.md`
- Advanced driver overlap: `crates/fret-launch/src/runner/common/winit_app_driver.rs`
- Function-pointer driver surface: `crates/fret-launch/src/runner/common/fn_driver.rs`
- Desktop app builder surface: `ecosystem/fret/src/app_entry.rs`
- Wrapped driver/builder surface: `ecosystem/fret/src/lib.rs`
- Manual assembly facade: `crates/fret-framework/src/lib.rs`
- Golden path contract framing: `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- Runtime contract boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- GPUI reference surface: `repo-ref/zed/crates/gpui/src/app.rs`
- GPUI crate root exports: `repo-ref/zed/crates/gpui/src/gpui.rs`

## Open questions

1. Should `fret-launch` keep a public `runner` module at all, or only curated root re-exports?
2. Do we want a future `LaunchOptions`-style public wrapper around `WinitRunnerConfig`, or is staged curation enough?
3. Should web gain a peer high-level entry surface in `fret`, or should the docs remain explicit that the current top-level builder is desktop-first?
4. Which currently exported launch helper types are truly contract-worthy versus merely convenient today?

