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

### Bootstrap Plan Gates

```bash
cargo tree -p fret-bootstrap --no-default-features -e normal --depth 4
cargo check -p fret-bootstrap --no-default-features
```

What this proves:

- Backend-free bootstrap policy/default construction remains separate from concrete launch/render
  adapters.

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
