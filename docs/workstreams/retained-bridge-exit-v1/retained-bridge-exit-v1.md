# Retained Bridge Exit Plan v1 (Fearless Refactor)

Status: Closed for the runtime retained-bridge exit (workstream notes only; ADRs remain the source
of truth)

Tracking file:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Related references:

- `docs/architecture.md` (declarative mount + retained semantics)
- `docs/runtime-contract-matrix.md` (mechanism-only runtime contract map)
- `docs/adr/0066-fret-ui-runtime-contract-surface.md` (Stable/Experimental/Compatibility tiers)
- `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md` (why the bridge exists)
- `tools/check_layering.py` (layering + feature allowlists)

## 1) Why this workstream exists

Fret’s primary UI direction is **declarative authoring** (per-frame element tree) mounted into a runtime
substrate that provides **retained semantics** (focus/IME correctness, caching, virtualization, overlay layering).

However, some ecosystem surfaces (docking, node graphs, charts/plots) were historically authored as retained
widgets. To migrate policy-heavy UI out of `crates/fret-ui` without a rewrite, the runtime originally exposed a
feature-gated compatibility mechanism. That mechanism has now exited the runtime:

- `fret-ui/unstable-retained-bridge` (module: `fret_ui::retained_bridge`) was deleted by
  `RBX-M4-043`.
- retained widget context exports and `UiTreeRetainedExt` moved out of the bridge/facade path or
  were deleted before `RBX-M4-043`.
- `ElementKind::RetainedSubtree` (a declarative leaf that hosted a retained subtree) was removed by
  `RBX-M4-014` after no runtime users remained.

This bridge is intentionally sharp and temporary. Without an explicit exit plan, it risks becoming a permanent
authoring path that:

1. fragments the ecosystem authoring model,
2. expands unstable runtime surface area over time,
3. makes “core vs ecosystem” extraction harder,
4. and weakens the incentive to finish declarative primitives.

This workstream defines a **fearless refactor** plan to shrink and eventually remove the retained bridge (or at
least make it trivially removable) while keeping editor-grade demos moving.

## 2) Scope and non-goals

In scope:

- Keep `crates/fret-ui` mechanism-only; prevent policy creep through bridge expansion.
- Limit where `unstable-retained-bridge` can be enabled (allowlist + CI gate); after `RBX-M4-043`,
  the allowlist is empty and the gate is a no-regrowth policy.
- Migrate high-value ecosystem UI to declarative authoring where feasible.
- Provide a staged plan so refactors remain safe and measurable.

Non-goals:

- Replacing the runtime substrate (`UiTree`) in this workstream. This is about **authoring + policy layering**.
- Guaranteeing full compatibility for bridge users. This is pre-1.0 “fearless refactor” mode.
- Solving every performance problem up-front (but we do add gates/diagnostics to avoid regressions).

## 3) Invariants (do not break)

These are “hard seams” (ADR-driven) that the exit plan must preserve:

1. **Mechanism vs policy split**
   - `crates/*` remain portable mechanisms and stable contracts.
   - policy-heavy components and interaction outcomes live in `ecosystem/*`.

2. **Declarative-only ecosystem golden path**
   - `ecosystem/fret-ui-shadcn` and shadcn-aligned components must not enable `unstable-retained-bridge`.
   - No workspace crate may enable `fret-ui/unstable-retained-bridge`; the feature has been deleted.
   - Node-local retained widget implementation code can remain only behind
     `fret-node/compat-retained-canvas` and must consume stable `fret-ui` root exports, not a runtime
     bridge facade.

3. **Bridge remains unstable + minimal**
   - There is no remaining `fret_ui::retained_bridge` or `fret_ui::compat_retained_canvas` surface.
   - Any attempt to reintroduce `fret-ui/unstable-retained-bridge` is a regression caught by
     `tools/check_layering.py`.

4. **Multi-window + overlays remain correct**
   - focus/IME, overlay dismissal, internal drag routing, and viewport input forwarding must keep working during
     and after migrations.

## 4) Strategy (how we get out)

### 4.1 Lock down the blast radius

Treat `unstable-retained-bridge` like a hazardous material:

- Add CI gates that:
  - reject `crates/* -> ecosystem/*` reverse deps,
  - restrict which crates may enable `fret-ui/unstable-retained-bridge`.

### 4.2 Prefer declarative “leaf primitives” over retained subtrees

For editor-grade surfaces, we often do not need retained widget authoring if we have the right declarative leaf
primitives:

- `Canvas` for custom scene emission (ADR-aligned).
- `ViewportSurface` for Tier A embedding (engine viewports).
- input/event region elements (`PointerRegion`, `InternalDragRegion`, `TextInputRegion`).
- action hooks (ADR 0074) for press/dismiss/roving/typeahead/timer policies.

The migration goal is not “rewrite everything as tiny elements”; it is “make policy-heavy ecosystems expressible
as declarative composition using stable mechanism primitives”.

### 4.3 Migrate the highest-value bridge clients first

Suggested priority order (subject to current roadmap priorities):

1. Docking UI (`ecosystem/fret-docking`) because it is the editor-grade backbone and a policy hotspot.
2. Node graph canvas (`ecosystem/fret-node`) because it exercises overlays + selection + input arbitration.
3. Charts/plots because they can often become `Canvas`-first.

Each migration should remove at least one of:

- a direct `Widget` implementation in ecosystem,
- a dependency on `UiTreeRetainedExt`,
- a usage of `RetainedSubtreeProps`.

## 5) Exit criteria (definition of done)

We consider the workstream successful when all of the following are true:

1. `ecosystem/fret-ui-shadcn` does not enable `unstable-retained-bridge` (and CI enforces this).
2. The allowlist of bridge-enabled crates is empty.
3. The runtime (`crates/fret-ui`) contains no retained bridge facade or `unstable-retained-bridge`
   feature; remaining node-local compatibility code uses stable `fret-ui` root exports.
4. We have at least one editor-grade demo (docking + viewports) implemented declaratively end-to-end.

All four criteria are satisfied as of `RBX-M4-043`. The broader architectural goal of splitting the
remaining node graph implementation into declarative chrome/overlays/panels plus a canvas/viewport
leaf is intentionally tracked as a follow-on, because it no longer blocks deletion of the runtime
retained bridge.

## 6) Risks and mitigations

- Risk: declarative primitives are missing for a migration target.
  - Mitigation: add mechanism-only primitives to `crates/fret-ui` behind an Experimental tier, with explicit ADR
    hooks and tests, rather than expanding policy in the bridge.

- Risk: performance regressions when moving from retained widget caches to declarative composition.
  - Mitigation: add `fretboard-dev diag` scripts and perf gates (window telemetry, cache-hit stats, invalidation
    hotspots) for the migrated surfaces.
