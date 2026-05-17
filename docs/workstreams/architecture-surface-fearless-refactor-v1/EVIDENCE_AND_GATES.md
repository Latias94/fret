# Architecture Surface Fearless Refactor v1 — Evidence And Gates

Status: Active
Last updated: 2026-05-17

## Baseline Observations

Commands run during the opening audit:

```bash
python tools/check_layering.py
python tools/check_consumption_profiles.py
cargo tree -p fret --no-default-features -e normal --depth 3
cargo tree -p fret --no-default-features --features app -e normal --depth 3
cargo tree -p fret-bootstrap --no-default-features -e normal --depth 3
```

Results:

- `tools/check_layering.py` passed.
- `tools/check_consumption_profiles.py` passed.
- `fret --no-default-features` still pulled backend/render dependencies including `fret-launch`,
  `fret-render`, `wgpu`, `winit`, `fret-platform-native`, and `fret-runner-winit`.
- `fret --no-default-features --features app` still pulled the same launch/render stack.
- `fret-bootstrap --no-default-features` still pulled `fret-launch`, `fret-render`, `wgpu`, `winit`,
  native platform, and runner crates.

## Gate Set

### Always Run For This Lane

```bash
python tools/check_layering.py
python tools/check_consumption_profiles.py
```

What this proves:

- Core and ecosystem dependency rules still hold.
- Existing modular consumption profiles still compile.

### Minimal `fret` Profile Gates

```bash
cargo tree -p fret --no-default-features -e normal --depth 4
cargo tree -p fret --no-default-features --features app -e normal --depth 4
cargo check -p fret --no-default-features
cargo check -p fret --no-default-features --features app
cargo check -p fret --no-default-features --features app --test backend_free_app_authoring_profile
```

What this proves:

- Backend-free app-authoring profiles do not silently pull launch/render/platform backend stacks.
- The documented app-authoring profile compiles.

Disallowed dependency names for backend-free profiles unless explicitly reclassified:

- `wgpu`
- `winit`
- `fret-launch`
- `fret-render`
- `fret-platform-native`
- `fret-runner-winit`

2026-05-17 result for ASF-020:

- `fret-bootstrap` and `fret-launch` were made optional in `ecosystem/fret/Cargo.toml`.
- `desktop` now explicitly owns `fret-framework/native-wgpu`, `fret-bootstrap`, `fret-launch`, and
  `wgpu`.
- `app` stays as the backend-free shadcn authoring baseline.
- `diagnostics`, `tracing`, `devloop`, `ui-assets`, `icons`, `preload-icon-svgs`, and
  `command-palette` are documented and gated as desktop-bound convenience features.
- `tools/check_consumption_profiles.py` now fails if the backend-free `fret` trees contain `wgpu`,
  `winit`, `fret-launch`, `fret-render`, `fret-platform-native`, or `fret-runner-winit`.
- Targeted checks passed:
  - `cargo fmt --package fret`
  - `python tools/check_layering.py`
  - `cargo tree -p fret --no-default-features -e normal --depth 4 --prefix none`
  - `cargo tree -p fret --no-default-features --features app -e normal --depth 4 --prefix none`
  - `cargo check -p fret --no-default-features`
  - `cargo check -p fret --no-default-features --features app`
  - `python tools/check_consumption_profiles.py`
  - `python tools/check_workstream_catalog.py`

2026-05-17 result for ASF-021:

- `FretApp` is now exported in backend-free app-authoring profiles instead of being wholly hidden
  behind `desktop`.
- Backend-free methods on `FretApp` are limited to authoring/spec configuration such as
  `new(...)`, defaults, config-file preference, static asset registrations, and setup bundles.
- Desktop execution methods remain `desktop`-only: window configuration, `view::<V>()`,
  `view_with_hooks::<V>(...)`, asset startup/reload policies, command-palette driver wiring,
  `UiAppBuilder`, and `.run()`.
- `ecosystem/fret/tests/backend_free_app_authoring_profile.rs` proves that
  `fret::app::prelude::*` exposes the backend-free `FretApp` authoring spec under
  `--no-default-features --features app`.
- `tools/check_consumption_profiles.py` now compiles that test target as part of the modular
  consumption gate.
- Targeted checks passed:
  - `cargo fmt --package fret`
  - `cargo check -p fret --locked --no-default-features`
  - `cargo check -p fret --locked --no-default-features --features app`
  - `cargo check -p fret --locked --no-default-features --features app --test backend_free_app_authoring_profile`
  - `cargo check -p fret --locked`
  - `python tools/check_consumption_profiles.py`
  - `cargo nextest run -p fret --locked --no-default-features --features app --test backend_free_app_authoring_profile`
  - `cargo nextest run -p fret --locked authoring_surface_policy_tests`

2026-05-17 result for ASF-031:

- `fret --no-default-features --features app` now exposes backend-free asset startup planning
  values through `fret::assets` and records them on `FretApp::asset_startup(...)` /
  `FretApp::asset_reload_policy(...)` without pulling launch/render/native backend crates.
- `desktop` remains an independent explicit runner/render opt-in; it can still apply the same
  planning values through `UiAppBuilder::with_asset_startup(...)` without implying the `app`
  feature or shadcn authoring baseline.
- First-party scaffolded asset modules keep generated startup application on
  `generated_assets::mount(builder)?`, which calls `UiAppBuilder::with_asset_startup(...)`; README
  guidance now distinguishes app-spec recording from desktop-builder application.
- The generated first-contact templates were migrated off stale `AppUi` call sites found by the
  scaffold compile gate: `cx.app` field access became `cx.app()`, and the Todo filter item text now
  uses `ui::text(...).into_element_in(cx)` instead of `cx.text(...)` on `AppUi`.
- Targeted checks passed:
  - `cargo fmt --package fret --package fretboard`
  - `cargo check -p fret --locked --no-default-features -j 1`
  - `cargo check -p fret --locked --no-default-features --features app -j 1`
  - `cargo check -p fret --locked --no-default-features --features desktop -j 1`
  - `cargo check -p fret --locked --no-default-features --features app --test backend_free_app_authoring_profile -j 1`
  - `cargo nextest run -p fret --locked --no-default-features --features app --test backend_free_app_authoring_profile -j 1`
  - `cargo nextest run -p fret --locked authoring_surface_policy_tests -j 1`
  - `cargo nextest run -p fretboard --locked assets -j 1`
  - `cargo nextest run -p fretboard --locked scaffold -j 1 --no-fail-fast`
  - `python tools/check_consumption_profiles.py`
  - `python tools/check_layering.py`
  - `python tools/check_workstream_catalog.py`

2026-05-17 result for ASF-040:

- `fret::app::prelude::*` now has a source-level closed pub-use budget in
  `ecosystem/fret/src/lib.rs`, covering both named exports and anonymous extension traits.
- Named prelude exports are limited to first-contact app authoring nouns: `FretApp`, `App`,
  `AppRenderContext`, `AppRenderCx`, `AppUi`, `Ui`, `UiChild`, `WindowId`, `View`, `Px`, `ui`, and
  optional `shadcn`.
- Extension-trait imports remain anonymous budget entries for grouped app action/data helpers,
  tracked state observation, feature-gated query/mutation read helpers, element conversion, style
  refinement, and a11y/test-id/semantics refinements.
- `docs/crate-usage-guide.md` now states that the app prelude is a closed Golden Path budget rather
  than a staging area; new styling, command, asset, environment/adaptive, router, docking, editor,
  activation, raw model, and low-level mechanism surfaces must stay on explicit modules unless that
  budget is deliberately revised.
- Targeted checks passed:
  - `cargo fmt --package fret`
  - `cargo nextest run -p fret --locked app_prelude_pub_use_budget_is_curated_and_closed -j 1`
  - `cargo nextest run -p fret --locked authoring_surface_policy_tests -j 1`

2026-05-17 result for ASF-041:

- `LocalState`, `LocalStateTxn`, `LocalActionCapture`, `WatchedState`, `TrackedStateExt`, and the
  LocalState-backed component model adapters moved from the monolithic `ecosystem/fret/src/view.rs`
  into the private owner module `ecosystem/fret/src/view/local_state.rs`.
- `crate::view` keeps stable re-exports for the existing app/advanced surfaces, so callers still
  use the same public paths while `view.rs` no longer owns the LocalState implementation body.
- Source-level tests now combine `view.rs` with `view/local_state.rs` for authoring-surface checks,
  which locks the public contract without treating single-file placement as the contract.
- Targeted checks passed:
  - `cargo fmt --package fret`
  - `cargo check -p fret --locked -j 1`
  - `cargo check -p fret --locked --features state,state-mutation -j 1`
  - `cargo nextest run -p fret --locked view::tests -j 1 --no-fail-fast`
  - `cargo nextest run -p fret --locked --test app_render_actions_surface --test app_render_data_surface --test render_authoring_capability_surface --test raw_state_advanced_surface_docs -j 1 --no-fail-fast`
  - `cargo nextest run -p fret --locked authoring_surface_policy_tests -j 1`

### Bootstrap Plan Gates

```bash
cargo tree -p fret-bootstrap --no-default-features -e normal --depth 4
cargo check -p fret-bootstrap --no-default-features
cargo check -p fret-bootstrap --no-default-features --test backend_free_bootstrap_profile
```

What this proves:

- Backend-free bootstrap policy/default construction remains separate from concrete launch/render
  adapters.

2026-05-17 result for ASF-030:

- `fret-bootstrap --no-default-features` no longer pulls `fret-launch`, `fret-render`, `wgpu`,
  `winit`, `fret-platform-native`, or `fret-runner-winit`.
- `fret-bootstrap` now owns a backend-free `assets` planning/default vocabulary and converts it to
  `fret-launch` only behind the explicit `launch` feature.
- `BootstrapBuilder`, UI app driver, diagnostics, hotpatch, and preload-on-GPU helpers are on the
  launch adapter lane; backend-free callers can still construct asset startup plans and reload
  policy values.
- `tools/check_consumption_profiles.py` now checks both the backend-free dependency tree and the
  backend-free public planning test.
- Targeted checks passed:
  - `cargo fmt --package fret-bootstrap`
  - `cargo check -p fret-bootstrap --locked --no-default-features`
  - `cargo tree -p fret-bootstrap --locked --no-default-features -e normal --depth 4 --prefix none`
  - `cargo check -p fret-bootstrap --locked --features launch`
  - `cargo check -p fret-bootstrap --locked --features ui-app-driver`
  - `cargo check -p fret-bootstrap --locked --all-features`
  - `cargo check -p fret-bootstrap --locked --no-default-features --test backend_free_bootstrap_profile`
  - `cargo nextest run -p fret-bootstrap --locked --no-default-features --test backend_free_bootstrap_profile`
  - `cargo test -p fret-bootstrap --locked --doc --no-default-features`
  - `cargo check -p fret-bootstrap --locked`
  - `cargo check -p fret --locked`
  - `python tools/check_consumption_profiles.py`
  - `python tools/check_layering.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`

### Facade Surface Gates

```bash
cargo nextest run -p fret
```

Use narrower filters when iterating on one public-surface family. Record the exact command next to
the task result.

What this proves:

- Public prelude, app entry, local state, action, selector/query, and advanced escape hatch tests
  still match the intended surface.

### Ecosystem Taxonomy Gates

```bash
python tools/check_layering.py
cargo nextest run -p fret-ui-kit
cargo nextest run -p fret-ui-shadcn
```

Use targeted filters during iteration. The closeout gate may be narrower if only one primitive
family is moved; record the reason.

What this proves:

- Shared policy modules stay backend-free.
- Recipe surfaces still pass behavior tests after moving ownership.

2026-05-17 result for ASF-050:

- Chosen representative family: boolean controls (`checkbox` + `switch`).
- `ecosystem/fret-ui-headless/src/boolean_control.rs` now owns pure optional-bool transitions for
  checkbox and switch. `CheckedState` remains the headless tri-state value in
  `ecosystem/fret-ui-headless/src/checked_state.rs`.
- `ecosystem/fret-ui-kit/src/primitives/{checkbox.rs,switch.rs}` no longer owns or re-exports the
  pure optional-bool helpers; those files keep runtime/a11y/controlled-model facades.
- First-party recipe/application consumers now import the headless owner directly where they need
  pure state: `ecosystem/fret-ui-shadcn/src/{checkbox.rs,switch.rs}`,
  `ecosystem/fret-ui-material3/src/checkbox.rs`,
  `ecosystem/fret-ui-editor/src/controls/checkbox.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/checkbox/table.rs`, and
  `ecosystem/fret/src/view/local_state.rs`.
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` now records that v1 does not reintroduce the deleted
  `fret-ui-primitives` crate; `fret-ui-kit::headless` remains a convenience re-export while direct
  headless ownership is the finalized path for this family.
- Targeted checks passed:
  - `cargo fmt --package fret-ui-headless --package fret-ui-kit --package fret-ui-shadcn`
  - `cargo fmt --package fret-ui-material3 --package fret-ui-editor`
  - `cargo fmt --package fret --package fret-ui-gallery`
  - `cargo test -p fret-ui-headless --locked --lib boolean_control -j 1`
  - `cargo check -p fret-ui-kit --locked -j 1`
  - `cargo check -p fret-ui-shadcn --locked -j 1`
  - `cargo check -p fret-ui-material3 --locked -j 1`
  - `cargo check -p fret-ui-editor --locked -j 1`
  - `cargo check -p fret --locked --no-default-features --features app -j 1`
  - `cargo check -p fret-ui-gallery --locked -j 1`
  - `cargo check -p fret --locked -j 1`
  - `cargo test -p fret-ui-kit --locked --lib primitives::checkbox -j 1`
  - `cargo test -p fret-ui-kit --locked --lib primitives::switch -j 1`
  - `cargo test -p fret-ui-shadcn --locked --lib checkbox_optional_none_is_indeterminate_and_toggles_to_checked -j 1`
  - `cargo test -p fret-ui-shadcn --locked --lib switch_optional_none_toggles_to_some_true -j 1`
  - `cargo test -p fret-ui-material3 --locked --lib checkbox_new_optional_controllable_applies_default_checked -j 1`
  - `python tools/check_layering.py`
  - `python tools/check_consumption_profiles.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`
- Note: `cargo nextest run -p fret-ui-headless --locked boolean_control -j 1 --no-fail-fast`
  was attempted first, but Windows refused to enumerate an unrelated integration-test binary with
  `os error 740`; the same owner tests were run through `cargo test --lib` instead.

2026-05-17 result for ASF-051:

- Chosen recipe surface: `fret-ui-shadcn::carousel`.
- `ecosystem/fret-ui-shadcn/src/carousel.rs` now imports
  `fret_ui_headless::{carousel, embla, snap_points}` directly instead of routing pure carousel
  engines through `fret_ui_kit::headless`.
- `docs/audits/shadcn-carousel.md` and `docs/adr/IMPLEMENTATION_ALIGNMENT.md` record the owner path.
- Targeted checks passed:
  - `cargo fmt --package fret-ui-shadcn`
  - `cargo check -p fret-ui-shadcn --locked -j 1`
  - `cargo test -p fret-ui-shadcn --locked --test carousel_loop_downgrade_without_embla_engine -j 1`
  - `python tools/check_layering.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`

### Renderer Facade Gates

The exact gate depends on the ASF-070 decision:

- collapse path: compile first-party renderer callers after dependency migration.
- deepen path: compile the chosen renderer profile and run renderer contract tests.

Record the chosen commands in this file when ASF-070 starts.

## Evidence Anchors

- `docs/workstreams/architecture-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/architecture-surface-fearless-refactor-v1/TODO.md`
- `docs/workstreams/architecture-surface-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/framework-modularity-fearless-refactor-v1/design.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0092-crate-structure-core-backends-apps.md`
- `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- `docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
