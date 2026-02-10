# Workspace Crate Boundaries Audit v1 (Render / Web Runner / Facades)

Status: Draft (workstream notes only; ADRs remain the source of truth)

Tracking file:

- `docs/workstreams/workspace-crate-boundaries-v1-todo.md`

Primary references:

- `docs/adr/0093-crate-structure-core-backends-apps.md`
- `docs/adr/0091-platform-backends-native-web.md`
- `docs/repo-structure.md`
- `docs/dependency-policy.md`
- `tools/check_layering.py`
- Upstream reference snapshots (notes only): `repo-ref/zed/`, `repo-ref/dioxus/`

## 1) Why this workstream exists

Fret already enforces the *direction* of dependencies well (kernel -> backends/apps; ecosystem remains extractable),
but a few high-leverage boundaries are still easy to blur over time:

1. **Rendering**: `crates/fret-render` currently bundles both “portable render contracts” and “wgpu implementation”.
   This makes future backend work (WebGPU-first wasm, alternative renderers, renderer refactors) harder than it
   needs to be.
2. **Web runner**: wasm input/event wiring currently lives under the winit runner, while `fret-runner-web` is a
   re-export shim. This weakens the long-term “dedicated DOM adapter for IME/keyboard fidelity” direction.
3. **Facade bundles**: `crates/fret` feature bundles should express the intended golden paths clearly (desktop vs web),
   without forcing one runner abstraction on all targets.

This workstream defines a staged, measurable refactor plan that improves long-term maintainability without
destabilizing demos.

## 2) Scope and non-goals

In scope:

- Reshape crates to keep contracts portable and implementations replaceable.
- Make the web direction explicit: a dedicated web adapter (`fret-runner-web`) as the default wasm path.
- Keep the public facade (`crates/fret`) coherent with ADR 0091/0093.
- Remove the layout engine feature fork if it is no longer a real decision point (reduce test matrix).

Non-goals:

- Redesigning the UI runtime contracts (`crates/fret-ui`) beyond what is required by the boundary refactor.
- Solving all web UX gaps (focus/IME/pointer-lock/etc) in this workstream; the goal is *correct seams*.
- Rewriting ecosystem policy crates; only touch them when a dependency migration requires it.

## 3) Invariants (do not break)

These rules must remain true throughout the refactor:

1. **Kernel portability**
   - `crates/fret-core`, `crates/fret-runtime`, `crates/fret-app`, `crates/fret-ui` remain backend-agnostic.
2. **Ecosystem extractability**
   - `ecosystem/*` must not depend on backend crates unless explicitly allowlisted by policy.
3. **No reverse edge: kernel -> ecosystem**
   - `crates/*` must not depend on `ecosystem/*`.
4. **Backends do not depend on UI/components**
   - `fret-platform-*`, `fret-runner-*`, and renderer backends must not depend on component/policy crates.

## 4) Proposed changes (high level)

### 4.1 Render: split “contract/core” from “wgpu backend”

Target shape:

- `crates/fret-render-core`: backend-agnostic render contracts + data types.
- `crates/fret-render-wgpu`: wgpu implementation (pipelines, uploads, text/svg rasterization, etc).
- `crates/fret-render`: compatibility facade (short-term) to reduce churn; can become a small meta crate later.

Migration strategy:

1. Introduce the new crates with minimal code movement (re-export first if needed).
2. Move implementation modules into `fret-render-wgpu` incrementally.
3. Update `crates/fret-launch` to depend on `fret-render-wgpu` (not the contract crate).
4. Keep `crates/fret` bundles stable via `fret-render` facade until a planned breaking release.

### 4.2 Web: make a real `fret-runner-web` adapter (DOM + RAF + canvas wiring)

Target shape:

- `crates/fret-runner-web`: a real adapter that:
  - listens to DOM/canvas events,
  - maps them into `fret-core::Event`,
  - provides RAF scheduling hooks,
  - remains independent from `winit` (default direction).
- `crates/fret-runner-winit`: returns to “winit-only” responsibilities (desktop-first).

Bundle intent:

- `crates/fret`:
  - `web` bundle should use `fret-runner-web` (default wasm path).
  - An optional `web-winit` (or `wasm-winit`) bundle can exist for compatibility experiments.

### 4.3 Layout: delete the “feature fork” if the decision is already final

If `taffy` is the committed direction, remove the now-stale feature fork to:

- shrink compile/test matrix,
- avoid dead code paths,
- and make layout behavior easier to reason about.

### 4.4 Router/query: consolidate if it improves user cognition

Decision: keep `fret-router` and `fret-query` as separate ecosystem crates.

Rationale:

- `fret-router` is not “just glue”: it already provides a portable URL/route model (path patterns,
  canonicalization, in-memory history) plus an opt-in wasm adapter.
- `fret-query` is an async resource state/cache layer (TanStack Query-like) that should remain useful
  outside of navigation concerns.

Boundary tightening follow-ups:

- Keep route↔query helpers behind an explicit feature (today: `fret-router/query-integration`).
- Keep `fret-query` portable by default by requiring callers that want `ElementContext` sugar to opt
  into `fret-query/ui` explicitly (avoid “UI by default” in transitive deps).
- Apply the same “portable by default” rule to selector helpers (`fret-selector/ui`), so pure
  derived-state memoization remains usable without pulling UI/runtime deps transitively.

## 5) Definition of done

This workstream is considered complete when:

1. Render backend swap is feasible without contract churn:
   - `fret-render-core` contains portable contracts only;
   - `fret-render-wgpu` contains wgpu implementation only.
2. Web path is coherent and explicit:
   - `fret-runner-web` is a real adapter;
   - `fret-runner-winit` no longer hosts DOM-specific glue;
   - `fret` facade `web` bundle follows the default direction.
3. Layout feature fork is removed (if committed) and the workspace builds cleanly.
4. `tools/check_layering.py` remains green and CI-like guardrails remain in place.
