# App Entry Builder v1 (Refactor Design)

## Status snapshot

This workstream is now **partially implemented** and is the recommended desktop-first onboarding
surface for the `fret` facade.

Current recommended entry paths:

- `fret::FretApp::new(...).window(...).view::<V>()?`
- `fret::FretApp::new(...).window(...).view_with_hooks::<V>(...)?`

Update (2026-03-10):

- `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` were removed from `fret` before the first
  published release.

Companion naming decisions:

- Primary public type: `fret::FretApp`
- Canonical app import: `use fret::app::prelude::*;`
- Older top-level shorthand helpers were removed from `fret` during this fearless refactor.

Implementation anchor:

- `ecosystem/fret/src/app_entry.rs`

## Problem statement

Fret needs a compact app-facing entry surface that is ergonomic for general-purpose apps, while
still preserving the editor-grade seams that matter for real products:

- function-pointer lifecycle hooks by default,
- explicit bootstrap/install seams,
- advanced driver hooks for GPU/editor behavior,
- clean layering between framework mechanisms and ecosystem policy.

Before this workstream, the capabilities already existed, but the public story was split across
multiple helpers and lower-level concepts, which made onboarding noisier than necessary.

## Goals

- Make the recommended desktop-first path obvious from the crate root.
- Keep the underlying driver hotpatch-friendly by default (`fn` pointers, no captured closures).
- Preserve advanced seams on the `fret` facade so users do not need to drop to `fret-launch`
  immediately.
- Keep mechanism vs policy boundaries intact (`crates/*` vs `ecosystem/*`).
- Make docs/examples converge on one primary mental model.

## Non-goals

- Replacing `fret-bootstrap` or `UiAppDriver`.
- Moving policy-heavy behavior into `crates/fret-ui`.
- Forcing closure-based entry as the default posture.
- Redesigning the workspace crate graph in this workstream.

## Decisions

### 1) Ownership stays in `ecosystem/fret`

The builder belongs to the app-facing ecosystem facade, not the kernel/framework crates.

Why:

- it is an onboarding and composition surface,
- it bundles policy defaults,
- it should be free to evolve faster than the harder kernel contracts.

### 2) `FretApp` is the primary name

The builder is intentionally short and first-contact friendly:

```rust
use fret::app::prelude::*;

fn main() -> fret::Result<()> {
    FretApp::new("hello")
        .window("Hello", (560.0, 360.0))
        .view::<HelloView>()?
        .run()
}
```

To reduce confusion in code search and docs:

- the app surface teaches `FretApp`,
- the crate root now exports `FretApp` directly,
- and docs/examples should use `FretApp` plus `fret::app::prelude::*`.

### 3) The minimal builder surface is builder-first, not runner-first

The current builder surface is intentionally small but complete enough for general apps:

- `new(root_name)`
- `window(title, size)`
- `defaults(...)` / `minimal_defaults()` / `config_files(...)`
- `ui_assets_budgets(...)`
- `setup(...)`
- `setup_with(...)`
- icon-pack app installers via `setup(...::app::install)`
- `view::<V>()`
- `view_with_hooks::<V>(configure)`
- `run_view::<V>()` / `run_view_with_hooks::<V>(...)`

This is enough to keep the first-app story compact while leaving real seams available.

Hooks that need `UiServices`, GPU-ready customization, or custom effect installation remain
available only through explicit advanced builder extensions.

Pack-specific or raw registry helpers such as `register_icon_pack(...)` stay below the `fret`
facade on `fret-bootstrap`, where manual assembly is already explicit.

This naming convergence applies to the `fret` facade builder surface. The lower-level
`fret_bootstrap` raw/manual-assembly builders intentionally remain explicit and may keep older
mechanism-oriented method names such as `init_app(...)`.

### 4) Advanced seams stay available on the builder path

The key refinement in this round is that advanced driver configuration no longer requires dropping
back to the older top-level helpers.

`view_with_hooks(...)` keeps the following hooks reachable on the builder
path:

- event handling,
- engine-frame recording,
- viewport input,
- command handling,
- window create/close hooks,
- command palette opt-in,
- other `UiAppDriver` wiring seams already supported by `fret-bootstrap`.

This keeps `fret` suitable for both general-purpose apps and many editor-style apps.

### 5) The crate-root story is builder-only

The older top-level shorthand helpers have now been removed from `fret`.

That is intentional for this pre-release, fearless-refactor phase:

- the crate root now teaches a single mental model,
- docs/examples/templates no longer have to carry two competing first-contact paths,
- advanced seams remain available through the builder-preserving hook entry points.

## Why this shape works

### For general-purpose apps

The builder chain gives a short path with sane defaults:

- one root name,
- one main window declaration,
- one app/view entry choice,
- optional install hooks only when needed.

That is the right level of ceremony for ordinary desktop apps.

### For editor-grade apps

The builder does not hide the seams that actually matter:

- custom GPU work,
- multi-window lifecycle hooks,
- embedded viewport behavior,
- input interception and command routing,
- bootstrap-time installs.

So app authors can stay on `fret` longer before they need `fret-bootstrap` or `fret-launch`.

### For layering discipline

The builder remains a composition layer over existing `fret-bootstrap` / `UiAppDriver` machinery.
It does not move policy into the wrong crate and does not create a second runtime architecture.

## Rollout guidance

The recommended documentation order is now:

1. `fret::FretApp::new(...).window(...).view::<V>()?` for the default app-author entry.
2. `fret::FretApp::new(...).window(...).view_with_hooks::<V>(...)?` when driver customization is needed.
3. Drop to `fret-bootstrap` or `fret-framework` only when runner/manual assembly ownership is the
   actual need.

## Remaining questions

- Do we want to remove or archive any remaining historical docs that still teach pre-builder entry paths?
- Do we want an explicit closure-based entry later, and if yes, behind what naming/feature gate?
- Which additional conveniences deserve first-class builder methods versus remaining install-hook
  recipes?

### Optional “closure entry”

If we introduce a closure-based variant, it must be:

- opt-in via an explicit feature and/or type name (e.g. `ClosureApp`),
- documented as “not guaranteed hotpatch-friendly”.

## Migration / Compatibility

- Historical top-level helpers such as `fret::run` / `fret::app_with_hooks` are removed from the
  current `fret` surface.
- The builder chain is now the recommended author-facing app entry on `fret`.
- When app authors need lower-level ownership, docs should teach dropping down to
  `fret-bootstrap` / `fret-framework` rather than reviving parallel `fret` helpers.
- Templates (`fretboard new hello/simple-todo/todo`) and golden-path docs should stay on the
  builder chain.
- Keep docs showing both:
  - “recommended builder chain”
  - “drop down to `fret-bootstrap` for manual assembly”

## Open Questions

1) Naming: should the builder be `fret::App` or something less likely to confuse with
   `fret_app::App` (e.g. `fret::AppBuilder`, `fret::UiApp`)?
2) Where should the builder live?
   - `ecosystem/fret` only (recommended), or
   - also mirrored in `fret-bootstrap`?
3) How many “first-class knobs” are worth including before we push users to `install_*` hooks?
4) Do we want a single `Defaults::desktop_batteries()` preset, or multiple smaller presets?

## References (in-repo)

- Golden path driver contracts: `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- Hotpatch safety: `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Ecosystem bootstrap: `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Existing templates & onboarding ladder: `docs/examples/todo-app-golden-path.md`

## Evidence anchors

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/README.md`
- `docs/workstreams/app-entry-builder-v1/TODO.md`
- `docs/workstreams/app-entry-builder-v1/MILESTONES.md`
