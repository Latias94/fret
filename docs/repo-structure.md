# Repository Structure (Core vs Ecosystem)

This document describes the **intended** repository layout from a contributor/user cognition perspective.

Primary goal:

- Make it obvious what is **core framework** vs what is **ecosystem / likely-to-move**.
- Keep the core workspace approachable (Bevy-like “few top-level buckets”, minimal hierarchy).
- Preserve ADR 0037’s long-term direction: components can move to a separate `fret-components` repository later.

## Top-Level Buckets

Recommended top-level layout:

- `crates/`: **Core framework** crates (stable boundaries; framework + backends + runner glue).
- `ecosystem/`: **Ecosystem crates** incubated in-tree (components, icon sets, app kits). These are expected to evolve quickly and may move to another repository in the future.
- `apps/`: Runnable apps / end-to-end harness shells (not in `default-members`).
- `docs/`: Architecture docs + ADRs (source of truth).
- `tools/`: Scripts and maintenance utilities (e.g. layering checks).
- `repo-ref/`: Pinned upstream references (not build dependencies).
- `assets/`, `themes/`, `screenshots/`: Non-code assets.

Notes:

- `target/` and `.fret/` are generated and should remain out of VCS.
- Crate layer boundaries (core vs backends vs apps) are locked in `docs/adr/0093-crate-structure-core-backends-apps.md`.

## `crates/` (Core Framework)

`crates/` should contain the crates that define “what Fret is”:

- Contracts / portable core:
  - `fret-core`
  - `fret-runtime`
- Default integrated runtime:
  - `fret-app`
  - `fret-ui`
  - `fret-ui-app`
- Backends (desktop-first now, wasm/WebGPU later):
  - `fret-platform` (portable contracts)
  - `fret-platform-native` (desktop backend)
  - `fret-platform-web` (web backend)
  - `fret-runner-winit` (winit platform adapter: event mapping + input normalization)
  - `fret-render` (wgpu-based renderer)
- Integration / wiring:
  - `fret-launch` (cross-platform launcher glue; depends on backend crates)
- Public facade:
  - `fret` (re-exports)
- Other core glue:
  - `fret-a11y-accesskit`

Rule of thumb:

- If a crate is required to explain the core architecture in `docs/architecture.md`, it belongs in `crates/`.

## `ecosystem/` (In-Tree Incubation)

`ecosystem/` is for crates that are useful and real, but not part of the minimal framework kernel.

Common examples:

- Component/policy layers built on top of `fret-ui`:
  - `fret-ui-kit` (was `fret-ui-kit`)
  - `fret-ui-docking` (was `fret-ui-docking`)
  - `fret-ui-shadcn` (was `fret-ui-shadcn`)
- “App kit” / default app policies:
  - `fret-app-kit` (was `fret-app-kit`)
- Icons:
  - `fret-icons` (primitives/registry; was `fret-icons`)
  - `fret-icons-lucide` (icon set)
  - `fret-icons-radix` (icon set)

Long-term intent:

- `ecosystem/` can be extracted to a separate `fret-components` repository with minimal churn.

## Naming Guidelines (User Cognition)

Prefer names that encode **layer** and avoid ambiguous “sounds-like-core” labels.

- Use `fret-ui-*` for **UI components / policy-heavy layers** built on `fret-ui`.
  - Example: `fret-ui-docking` (explicitly UI-layer docking behavior)
  - Avoid: `fret-docking` (sounds like the core docking model/ops, which live in `fret-core`)
- Use `fret-app-*` for app-level conveniences and defaults that depend on `fret-app`.
- Use `fret-icons-*` for icons and icon sets.
- Keep backend crates explicit (`fret-platform-*`, `fret-render-*`, `fret-runner-*`).

## Cargo Workspace Membership

The workspace should include both buckets:

- `crates/*`
- `ecosystem/*`
- `apps/*`

This keeps demos and layering checks consistent while still communicating “core vs ecosystem” through the folder name.

## `apps/` (Runnable Harnesses)

`apps/` is for runnable apps and end-to-end harnesses that exercise the full stack.

Current apps:

- `fret-examples`: shared harness code (components gallery, docking demos, smoke tests).
- `fret-demo`: native harness shells (thin wrappers over `fret-examples`).
- `fret-demo-web`: wasm harness shell (Trunk + `#[wasm_bindgen(start)]`, thin wrapper over `fret-examples`).

## Extraction Policy (When to Move `ecosystem/` Out)

Indicators that a crate (or the whole `ecosystem/`) should move to a separate repository:

- Faster iteration cadence than core framework contracts.
- Frequent API churn that would be noisy in the main `fret` repo tags/releases.
- Desire to publish/ship optional components independently from core backends.

When extracting:

- Preserve crate names and public API paths whenever possible.
- Keep `fret` facade framework-only (per ADR 0037).
