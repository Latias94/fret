# GPUI Parity Refactor (Experience + Performance) — Workstream Plan

Status: Draft (design document for alignment)

This document proposes a “fearless refactor” plan to close the **experience** (interaction feel, authoring ergonomics)
and **performance** (CPU frame time stability, cache effectiveness, predictable invalidation) gap between:

- **Fret runtime substrate**: `crates/fret-ui`, `crates/fret-app`, `crates/fret-runtime`, `crates/fret-render`
- **Zed/GPUI reference substrate** (non-normative): `repo-ref/zed/crates/gpui`
- **gpui-component policy/recipes** (non-normative): `repo-ref/gpui-component/crates/ui`

This is a workstream note (not an ADR). Any “hard-to-change” contract changes must be captured as ADRs.

---

## 0. Executive Summary (What We Should Refactor, Not What We Should Add)

The gap you’re feeling is usually not “missing widgets”. It’s missing a **closed loop** between:

1) authoring model (how easy it is to express UI),
2) identity + state (what persists across frames),
3) observation + invalidation (what causes recomputation),
4) caching (what can be replayed safely),
5) introspection (how we debug correctness and performance).

Fret already has many of the building blocks:

- Declarative per-frame element tree + cross-frame element state (ADR 0028) implemented via `GlobalElementId -> NodeId` reuse
  - Fret anchors: `crates/fret-ui/src/declarative/mount.rs:113` (`render_root`), `crates/fret-ui/src/declarative/mount.rs:381` (`mount_element`)
- Element state store + GC lag frames
  - Fret anchors: `crates/fret-ui/src/elements/access.rs:13` (`with_element_state`), `crates/fret-ui/src/elements/runtime.rs:31` (`ElementRuntime`), `crates/fret-ui/src/elements/runtime.rs:48` (`set_gc_lag_frames`)
- Model observation + invalidation propagation (layout/paint record dependencies; model changes invalidate nodes)
  - Fret anchors: `crates/fret-ui/src/elements/cx.rs:304` (`observe_model_id`), `crates/fret-ui/src/tree/mod.rs:1269` (`propagate_model_changes`)
- Subtree replay caching for paint ops (ADR 0055) + per-window counters
  - Fret anchors: `crates/fret-ui/src/tree/mod.rs:632` (`ingest_paint_cache_source`), `crates/fret-ui/src/tree/mod.rs:589` (`set_paint_cache_policy`),
    `crates/fret-ui/src/tree/paint.rs:135` (replay), `crates/fret-ui/src/tree/mod.rs:110` (`UiDebugFrameStats`)

What GPUI adds (and what gives Zed its “feel”) is the *integration polish*:

- A first-class “view caching” authoring pattern (`AnyView::cached`) that reuses recorded prepaint/paint ranges when not dirty
  - GPUI anchors: `repo-ref/zed/crates/gpui/src/view.rs:103` (`AnyView::cached`), `repo-ref/zed/crates/gpui/src/view.rs:216` (`reuse_prepaint`),
    `repo-ref/zed/crates/gpui/src/view.rs:280` (`reuse_paint`)
- A path-based `GlobalElementId` (debuggable identity) built via `Window::with_global_id`
  - GPUI anchors: `repo-ref/zed/crates/gpui/src/window.rs:2039` (`with_global_id`), `repo-ref/zed/crates/gpui/src/window.rs:2861` (`with_element_state`)
- A single, consistent mental model for invalidation: “notify -> dirty views -> reuse ranges unless dirty/refreshing/inspecting”
  - GPUI anchors: `repo-ref/zed/crates/gpui/src/_ownership_and_data_flow.rs` (ownership + observe/notify narrative),
    `repo-ref/zed/crates/gpui/src/subscription.rs` (subscriber mechanics)

This plan focuses on refactoring Fret to gain:

- **View-level caching semantics** (GPUI-style) on top of the existing paint cache (ADR 0055), without bloating `fret-ui` into a kit.
- **Authoring density** improvements in ecosystem (`fret-ui-kit`/`fret-ui-shadcn`) so writing UI is “fluent” like gpui-component.
- **Debuggability** improvements (identity/inspector/perf HUD) so regressions are obvious.

---

## 1. Scope, Constraints, and Principles

### 1.1 Scope (in)

- Refactor experience/perf primitives in:
  - `crates/fret-ui` (runtime substrate, caching, routing, element bridge)
  - `crates/fret-app` / `crates/fret-runtime` (effects scheduling + model store integration)
  - `ecosystem/fret-ui-kit` / `ecosystem/fret-ui-shadcn` (authoring ergonomics + policy surfaces)
- Add targeted instrumentation and acceptance harnesses.

### 1.2 Out of scope (for this workstream)

- Replacing renderer architecture end-to-end (tracked elsewhere).
- New “big features” (complex widgets) unless required as perf/UX harness.
- API stabilization / public crate split changes unless they directly unblock parity.

### 1.3 Hard constraints (“fearless” does not mean “contractless”)

- Keep `crates/fret-ui` mechanism-only (ADR 0066): interaction policy stays in ecosystem.
- Preserve the “build every frame” contract for declarative roots (ADR 0028).
- Preserve ordering semantics (ADR 0002 / ADR 0009).
- Keep portability boundaries (`fret-runtime` effects, no `wgpu` types in UI runtime).

---

## 2. Target Outcomes (What “Parity” Means)

### 2.1 Experience targets (user-visible)

- Overlays: tooltips/popovers/menus feel stable (no flicker), dismissal and focus restore are consistent.
- Text: basic single-line and multi-line editing behave predictably under IME and high DPI.
- Lists: virtualized lists scroll smoothly, selection/cursor follow user expectations, and hover/press state is stable.
- Docking: cross-window drag and panel keep-alive avoid one-frame “holes”.

### 2.2 Performance targets (developer-visible, measurable)

We should define explicit acceptance thresholds (initial proposal):

- Idle frame CPU cost approaches “near-zero” (no full traversal/paint in steady state unless animating).
- Cache effectiveness is visible and trustworthy:
  - `UiDebugFrameStats.paint_cache_hits/misses/replayed_ops` trend upward on stable scenes (`crates/fret-ui/src/tree/mod.rs:110`).
- Large UI surfaces remain stable:
  - 10k-row virtual list: scrolling does not trigger full relayout of unrelated subtree.

---

## 3. Current-State Snapshot (Fret vs GPUI)

### 3.1 Fret declarative bridge (already GPUI-aligned)

- Element tree is rebuilt each frame for each root:
  - `crates/fret-ui/src/declarative/mount.rs:113` (`render_root`)
- Cross-frame state lives in `ElementRuntime`:
  - `crates/fret-ui/src/elements/access.rs:13` (`with_element_state`)
- Identity mapping avoids a reconcile engine:
  - `crates/fret-ui/src/declarative/mount.rs:381` (`mount_element` reuses `GlobalElementId -> NodeId`)
- Continuous frames are requestAnimationFrame-driven:
  - `crates/fret-runtime/src/effect.rs:161` (`Effect::RequestAnimationFrame`)
  - `crates/fret-ui/src/declarative/mount.rs:229` (push RAF effect when `wants_continuous_frames`)

### 3.2 Fret invalidation + caching (already has the seed, but not author-facing)

- Model observation is recorded during layout/paint, then used to propagate invalidation:
  - `crates/fret-ui/src/elements/cx.rs:304` (`observe_model_id`)
  - `crates/fret-ui/src/tree/mod.rs:1269` (`propagate_model_changes`)
- Paint cache is range-based and replays ops with translation (ADR 0055):
  - `crates/fret-ui/src/tree/paint.rs:135` (`replay_ops_translated`)
  - `crates/fret-ui/src/tree/mod.rs:632` (`ingest_paint_cache_source`)

### 3.3 GPUI’s “missing glue” (what we should emulate)

- Identity is path-based and debuggable:
  - `repo-ref/zed/crates/gpui/src/window.rs:2039` (`with_global_id`)
- Element state is staged across frames and guarded against reentrancy:
  - `repo-ref/zed/crates/gpui/src/window.rs:2861` (`with_element_state`)
- View caching is an explicit authoring pattern (`AnyView::cached`) with robust correctness gates:
  - `repo-ref/zed/crates/gpui/src/view.rs:103` (`cached`)
  - `repo-ref/zed/crates/gpui/src/view.rs:216` (`reuse_prepaint`)
  - `repo-ref/zed/crates/gpui/src/view.rs:227` (`detect_accessed_entities`)
  - `repo-ref/zed/crates/gpui/src/view.rs:280` (`reuse_paint`)

---

## 4. Refactor Strategy Overview (Phased, with Explicit Design Decisions)

### Phase 0 — Instrumentation First (1–2 weeks)

Goal: make it impossible to regress “feel” silently.

- Add a “Perf HUD” and/or tracing spans for:
  - layout time, paint time, engine solves, paint cache hit/miss (already in `UiDebugFrameStats`: `crates/fret-ui/src/tree/mod.rs:110`)
  - element-state GC churn (`gc_lag_frames` effects; `crates/fret-ui/src/elements/runtime.rs:48`)
  - model invalidation fan-out (how many nodes invalidated per model change; `crates/fret-ui/src/tree/mod.rs:1269`)
- Add two harness demos:
  1) Overlay torture test (popover + menu + tooltip + focus trap + outside press)
  2) Virtual list torture test (10k+ rows, variable heights, selection + hover + inline text input)

Deliverable: a repeatable “before/after” baseline (numbers, not vibes).

### Phase 1 — Authoring Density (ecosystem-only, 2–4 weeks)

Goal: writing UI should feel like gpui-component, while keeping runtime mechanism-only.

- Introduce a fluent authoring layer in `ecosystem/fret-ui-kit`:
  - “styled” helpers that translate to `LayoutStyle`/theme tokens.
  - `ElementExt` traits for common patterns (flex/stack/padding/margins/border/radius/shadows).
  - Mirror gpui-component patterns for discoverability:
    - Reference: `repo-ref/gpui-component/crates/ui/src/styled.rs`

This is the fastest “experience win” because it reduces boilerplate and forces consistency.

### Phase 2 — View-Level Caching (runtime + ecosystem, 3–6 weeks)

Goal: promote caching from an internal optimization to a **first-class composition tool**, similar to `AnyView::cached`.

This is the highest leverage performance refactor.

We implement a “cached subtree” primitive that:

- is keyed by stable identity (`GlobalElementId` / `NodeId`),
- captures **cache keys** (bounds/scale/theme) plus **dependency keys** (observed models/globals),
- reuses recorded ranges (ADR 0055) and reuses observation sets on hit,
- is automatically disabled in inspection/picking modes.

Proposed ecosystem-facing API surface (runtime internal may differ):

- `cx.cached(cache_key, |cx| -> Vec<AnyElement>) -> Vec<AnyElement>`
  - where `cache_key` is an explicit, typed key for “layout-affecting inputs”.

### Phase 3 — “Prepaint” + Multi-stream Frame Recording (optional but recommended, 4–8 weeks)

Goal: converge toward GPUI’s “request_layout / prepaint / paint” separation so that:

- interaction/semantics streams can be constructed deterministically,
- caching can reuse *more than paint ops*.

This aligns directly with ADR 0055’s “multi-stream” direction.

### Phase 4 — Text System & Editor-Grade Inputs (parallel track)

Goal: close the biggest “editor feel” gap (IME, caret geometry, font stacks).

- Implement font stack bootstrap + stable key propagation (ADR 0162).
- Make TextInput/TextArea integrate tightly with:
  - `TextFontStackKey` changes,
  - theme revisions,
  - cursor rect scheduling,
  - selection/caret stability under caching.

---

## 5. Module-by-Module Refactor Plan

### 5.1 Authoring & Composition (ecosystem)

#### Problem

Fret’s declarative authoring currently pushes developers toward “construct enums/props” directly.
This is correct but not *dense*. The resulting UI code is harder to scan and harder to keep consistent.

#### Reference

- gpui-component “styled density”: `repo-ref/gpui-component/crates/ui/src/styled.rs`

#### Proposal

In `ecosystem/fret-ui-kit`:

1) Add a fluent “style patch” surface:
   - Methods mirror Tailwind-ish semantics but remain typed.
   - Output is a `LayoutStyle` + “chrome style” patch.
2) Add “recipes” in `ecosystem/fret-ui-shadcn` that map shadcn taxonomy to these patches.

Key rule: `crates/fret-ui` remains mechanism-only.

#### Acceptance

- The demo UI code for a representative screen reduces ~30–50% line count.
- Styling is consistent (one source of truth for spacing/density).

#### Open questions

- Do we want proc-macros for derive/DSL, or keep everything as plain Rust methods?

---

### 5.2 Identity, Debuggability, and Inspector UX (runtime + ecosystem)

#### Problem

Fret’s `GlobalElementId(u64)` is great for performance and portability, but weak for:

- debugging (“what is this id?”),
- inspector “navigate-to-source”,
- parity with GPUI’s readable path ids.

#### Reference

- GPUI path ids: `repo-ref/zed/crates/gpui/src/window.rs:2039` (`with_global_id`)

#### Proposal (non-breaking)

1) Keep `GlobalElementId(u64)` as the stable runtime key.
2) Add an *optional* debug registry (feature-gated, e.g. `diagnostics`) that records:
   - callsite location (`Location::caller()`),
   - keyed hash inputs,
   - parent chain (a human-readable path for inspector only).
3) Extend `WindowElementDiagnosticsSnapshot` (already behind feature) to include:
   - focused/hovered elements (exists),
   - element id debug strings,
   - last bounds/visual bounds in a readable form.

#### Acceptance

- Given a hovered/focused element, the inspector can show a stable, human-readable id path.
- “unkeyed list reorder” warnings point to source callsite.

---

### 5.3 Observation + Invalidation: Make It “Closed Loop” (runtime)

#### Problem

Fret’s observation model is already robust, but it’s distributed across:

- element runtime’s “observed models/globals per root” (for declarative authoring),
- UiTree’s “observed_in_layout/paint” (for retained widget runtime).

This makes it harder to build “view caching” with a single story.

#### Anchors

- Element observation: `crates/fret-ui/src/elements/cx.rs:304` (`observe_model_id`)
- Invalidation propagation: `crates/fret-ui/src/tree/mod.rs:1269` (`propagate_model_changes`)

#### Proposal

Introduce a unified “dependency token” concept that both pipelines can speak:

- `DependencySet = { observed_models, observed_globals }`
- `DependencyRevision = hash(DependencySet + relevant key revisions)`

Then implement:

1) For retained widgets: dependency set is produced per node during layout/paint (already exists implicitly).
2) For declarative elements: dependency set is produced per root (already exists in `WindowElementState.observed_models`).
3) Standardize how caching consumes dependency sets:
   - Cache hit must retain the previous dependency set for the subtree, not drop it.

This mirrors GPUI’s “detect_accessed_entities” for cached views:

- `repo-ref/zed/crates/gpui/src/view.rs:227` (`detect_accessed_entities`)

#### Acceptance

- Cached subtrees continue to invalidate correctly on model changes (no “stale UI”).

---

### 5.4 View-Level Caching (runtime + ecosystem)

#### Problem

We have internal paint-cache (ADR 0055), but we lack an authoring-facing, composition-friendly caching boundary,
equivalent to GPUI’s `AnyView::cached` (`repo-ref/zed/crates/gpui/src/view.rs:103`).

Node-level caching is necessary but not sufficient for editor-grade UI:

- Large editor UIs benefit from caching *semantic subtrees* (view/panel level), not just widget nodes.
- The author needs a simple, intentional way to say “this subtree is expensive; cache it unless dependencies change”.

#### Current anchors

- Paint replay: `crates/fret-ui/src/tree/paint.rs:135`
- Cache key policy: `crates/fret-ui/src/tree/mod.rs:589` (`set_paint_cache_policy`)
- Cache ingestion: `crates/fret-ui/src/tree/mod.rs:632` (`ingest_paint_cache_source`)

#### Proposal A (recommended): `CachedSubtree` element (declarative)

Add a new declarative element kind, conceptually:

- `ElementKind::Cached(CachedProps { key: u64, policy: CachePolicy, ... })`

Behavior:

1) During render, the author wraps expensive content:
   - `cx.cached(key_inputs, |cx| children)`
2) The runtime creates/uses a dedicated `NodeId` boundary for the cached subtree root.
3) Cache key includes:
   - bounds.size, scale_factor, theme_revision (match ADR 0055)
   - plus “explicit cache key” from author (to cover content-mask/text-style changes if needed)
4) Dependency sets:
   - track observed models/globals for that subtree (unify with §5.3)
5) Inspection/picking disables caching:
   - consistent with `PaintCachePolicy::Auto` and GPUI inspector behavior.

Why declarative-first?

- Because it’s the closest analog to `AnyView::cached`, and it composes with your long-term authoring direction (ADR 0028).

#### Proposal B: Widget-only “cache boundary” (retained)

Alternative is to introduce a retained widget that acts as a cache boundary.
This can work, but it’s less aligned with the long-term declarative model and risks duplicating authoring patterns.

#### Acceptance

- In the “virtual list torture test”, non-visible panels remain cached while list scrolls.
- Hovering tooltips/menus does not blow away unrelated cached panels.

#### Open questions

- Should the cache boundary be opt-in only, or should we provide a default heuristic (like `PaintCachePolicy::Auto`)?

---

### 5.5 Event Dispatch, Default Prevention, and Action Availability (runtime)

#### Problem

Editor-grade “feel” depends on subtle consistency:

- capture vs bubble phase semantics,
- ability to prevent default focus changes on pointer down,
- action availability queries along the dispatch path (used by menus/palette/shortcuts).

GPUI has explicit dispatch phases:

- `DispatchPhase::Capture/Bubble` (`repo-ref/zed/crates/gpui/src/window.rs`).

Fret has capture and bubbling, but lacks a unified “default prevention” + action-availability contract.

#### Proposal

1) Formalize dispatch phases in `fret-ui`:
   - capture pass for “state cleanup” and outside-press observers,
   - bubble pass for normal handlers.
2) Add `prevent_default()` semantics for pointer down:
   - specifically to stop implicit focus shifts or parent activation.
3) Introduce “action availability” queries:
   - integrate with the command system in `fret-app`,
   - mirror gpui’s “is action available along dispatch path” mental model.

#### Acceptance

- Overlays no longer cause accidental focus steals.
- Keyboard shortcuts respect focus scopes consistently across windows and overlays.

---

### 5.6 Overlays, Dismissal, and Focus Restore (runtime substrate; policy in ecosystem)

#### Problem

Fret’s layering model is strong (multi-root overlays), but experience gaps appear when:

- focus needs to be restored predictably after dismissal,
- outside press dismissal interacts with docking drags and viewport capture,
- “initial focus before layout” is needed.

#### Proposal

1) Make overlay lifecycle hooks first-class:
   - `on_open`: capture focus snapshot + install focus trap policy (ecosystem)
   - `on_close`: restore focus or redirect focus deterministically
2) Ensure overlay anchor geometry uses *visual bounds*:
   - use `visual_bounds_for_element` for render-transform aware anchoring.
3) Provide a “policy harness” suite in ecosystem:
   - Radix-aligned dismissal/focus outcomes regression tests.

#### Acceptance

- Popover/menu focus behavior matches the reference stack (`docs/reference-stack-ui-behavior.md`).

---

### 5.7 Text System (runtime + renderer + platform)

#### Problem

Text is the most visible editor-grade subsystem. Missing font bootstrap + incomplete IME pipeline will dominate perceived gap.

#### References

- GPUI text system: `repo-ref/zed/crates/gpui/src/text_system.rs`
- Fret scheduling and cursor area effects:
  - `crates/fret-runtime/src/effect.rs:15` (`Effect`)

#### Proposal

Split into two refactor tracks:

1) **Font stack bootstrap**
   - Implement ADR 0162 (stable font stack key propagation).
   - Guarantee that changing font stack triggers relayout via `TextFontStackKey` dependency.
2) **IME + caret geometry correctness**
   - Ensure cursor rect effect is updated precisely when caret moves (including during preedit).
   - Add acceptance tests that replay IME sequences (even if partially mocked).

#### Acceptance

- IME acceptance checklist passes for representative cases.
- No “cursor jumping” under caching + high DPI.

---

### 5.8 Virtualization & Large Collections (runtime + ecosystem)

#### Problem

Virtualization must compose with:

- selection models,
- keyboard navigation,
- active-descendant semantics (cmdk-like),
- accessibility collection semantics (future).

gpui-component offers a composable range-driven API:

- `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`

Fret already has `virtualizer`-backed metrics:

- `crates/fret-ui/src/virtual_list.rs`

#### Proposal

1) Standardize a “virtual list row recipe” in ecosystem:
   - keyed rows by stable item key,
   - per-row hover/press/focus behaviors implemented in policy layer.
2) Ensure virtualization integrates with caching boundaries:
   - the list itself is “hot”; surrounding panels should remain cached.
3) Add a “table/tree” scaffolding doc and demo:
   - make the “rich row” pattern canonical.

#### Acceptance

- 10k-row list scroll stays smooth and does not repaint unrelated panels.

---

### 5.9 Renderer/Scene: Ordering, Batching, and Recording Fingerprints (renderer + runtime)

#### Problem

Even with UI caching, we can lose perf if we still:

- re-encode identical scenes,
- re-upload unchanged resources,
- break batching due to overly fine-grained ops.

ADR 0055 already mentions renderer-side encoding reuse by scene fingerprint.

#### Proposal

1) Make `SceneRecording::fingerprint` a first-class debug metric (HUD/tracing).
2) Provide per-pass stats:
   - ops count by kind, clip/push/pop counts, text blobs, images.
3) Tie cache boundaries to renderer reuse:
   - if UI paint cache hits, renderer fingerprint should remain stable (when no external surfaces change).

#### Acceptance

- Stable UI produces stable fingerprints across frames.

---

## 6. Key Design Decisions to Confirm (Choose Defaults)

### D1 — Where does “View caching” live?

Options:

1) Runtime primitive in `crates/fret-ui` (recommended): because caching must be deterministic and contract-level.
2) Ecosystem-only wrapper: easier initially, but risks “caching that breaks invalidation” due to lack of tight integration.

Recommendation: (1), but with a small surface area and policy-free semantics.

### D2 — Do we introduce an explicit `prepaint` phase?

Options:

1) Keep current (layout + paint) and extend paint cache only.
2) Add `prepaint` to build future interaction/semantics streams and enable broader reuse (GPUI-like).

Recommendation: start with (1) for Phase 2, but design Phase 2 APIs so Phase 3 can add prepaint without breaking.

### D3 — Identity: keep `GlobalElementId(u64)` or move to path ids?

Recommendation: keep `u64` for runtime, add debug registry for readability (feature gated).

### Suggested defaults (based on current repo maturity)

These defaults are optimized for “ship demos fast while raising the performance/feel ceiling”:

1) **Primary feel target**: pick **E (overall perf/jank)** as the mainline, and use **A (overlays)** as the first acceptance harness.
   - Rationale: Zed-like “smoothness” is mostly “default idle is cheap” + “cache is trustworthy”.
   - Exception: if the current north star is “code editor-grade authoring & IME”, then bring **B (text/IME)** ahead of A, but still keep E as the substrate work.
2) **API break budget**: do **ecosystem-first** authoring refactors; allow only **additive runtime** changes (no breaking public surface) until parity baselines are stable.
3) **Caching semantics**: implement view-level caching as **explicit opt-in** (`cx.cached(...)`) first; keep runtime node-level caching as-is (`PaintCachePolicy::Auto`).
4) **Inspector**: start with **HUD + debug picking + tracing/logging**, and postpone a full inspector UI until the caching/dispatch semantics are stable.

---

## 7. Migration Plan (Concrete, “No Big Bang”)

### Step A — Baseline harness

- Add two demos (overlay + virtual list) and record baseline stats.

### Step B — Ecosystem authoring

- Land fluent styling helpers in ecosystem and migrate the demos.

### Step C — Cached subtree primitive

- Implement `CachedSubtree` element kind and wire it to existing paint cache and dependency sets.
- Add regression tests:
  - cached subtree invalidates when a dependent model changes,
  - cached subtree does not repaint when unrelated models change,
  - caching disabled under inspection/picking.

### Step D — Prepaint/multi-stream (optional)

- Add prepaint pass behind a feature flag.

### Step E — Text bootstrap

- Implement ADR 0162 and re-run text/IME acceptance checks.

---

## 8. Questions for You (to lock the direction before coding)

1) **Primary “feel” target**: which one hurts most today?
   - A) overlays (dismiss/focus/portal), B) text/IME, C) lists/tables, D) docking/multi-window drag, E) overall perf/jank
2) **API break budget**: are we allowed to introduce a new authoring surface in ecosystem and migrate demos first, before touching runtime APIs?
3) **Caching semantics**: do you prefer explicit author opt-in (`cx.cached(...)`) only, or also an auto policy (like `PaintCachePolicy::Auto`)?
4) **Inspector story**: do you want a built-in inspector UI (like GPUI), or is “log + HUD + debug picking” enough for now?

---

## 9. Future “Big Break” Refactor Corridors (when breaking changes are allowed)

### Direction decision (no interfaces locked)

When a breaking-change window is available, the repository’s **v2 refactor north star** is:

- **Corridor A**: GPUI-aligned runtime pipeline — **view-level caching + explicit three-phase pipeline**
  (`request_layout` / `prepaint` / `paint`) + ADR 0055-style **multi-stream frame recording**.

This is a direction only. It intentionally does **not** lock concrete APIs or data structures.

This section exists specifically to avoid being “ADR-locked” into an implementation shape that later blocks
a necessary performance/experience redesign.

The idea is to keep ADRs locking **invariants/outcomes**, while reserving explicit “corridors” for large, planned
replacement refactors (v2/v3) without rewriting the whole ecosystem.

### 9.1 Triggers (when we should consider a big break)

We should treat these as objective signals that incremental refactors are no longer cost-effective:

- Sustained perf ceiling issues (e.g., editor-scale UIs cannot remain smooth even with caching boundaries and tuned invalidation).
- Architectural impedance mismatch (e.g., retained `UiTree` constraints block a GPUI-style `prepaint/paint` multi-stream model).
- Text pipeline constraints (font stack, shaping, IME) require deep changes across renderer/runtime/platform boundaries.
- Debuggability debt becomes a velocity killer (can’t attribute jank or correctness regressions reliably).

### 9.2 ADR evolution policy (how not to get locked)

We should treat ADRs as “contracts with evolution lanes”, not as “forever implementation”.

Recommended policy:

1) **Lock outcomes, not data structures**
   - Example: lock “ordering semantics” (ADR 0002/0009), not “how ops are stored”.
2) **Version hard-to-change contracts**
   - Prefer “v1/v2” explicitly in contract names and types (e.g., `*V1` structs already exist in the repo for keymap).
   - Allow ADRs to be superseded with a new ADR that states migration rules.
3) **Reserve escape hatches**
   - Keep IDs opaque (`GlobalElementId`, `TextBlobId`, etc.) so representations can change.
   - Keep effect boundaries data-driven (`Effect`) so scheduler/work loop can evolve.
4) **Ship experimental lanes behind feature flags**
   - Prototype the “next pipeline” in parallel, then migrate demos, then flip defaults.

This keeps the repo “decision-driven” without making early ADRs a permanent ceiling.

### 9.3 Corridor A — “UI Runtime v2”: unify around frame recording + view caching

This corridor keeps the current crate layering, but changes the runtime pipeline shape when breaks are allowed:

1) Introduce an explicit **three-phase pipeline**:
   - `request_layout` → `prepaint` → `paint`
   - Aligns with GPUI’s mental model and makes multi-stream recording natural
     - Reference: `repo-ref/zed/crates/gpui/src/element.rs` (request_layout/prepaint/paint pattern)
2) Promote ADR 0055’s “multi-stream frame recording” from conceptual to real:
   - Paint stream (already): `SceneOp`
   - Add interaction stream (hit regions/cursors/tab stops), and eventually semantics stream
3) Make **view-level caching** a first-class substrate mechanism (not just a node optimization):
   - Align with GPUI’s `AnyView::cached` semantics
     - Reference anchors: `repo-ref/zed/crates/gpui/src/view.rs:103` (cached), `repo-ref/zed/crates/gpui/src/view.rs:216` (reuse_prepaint), `repo-ref/zed/crates/gpui/src/view.rs:280` (reuse_paint)
4) Unify dependency tracking into a single “closed loop”:
   - “what was accessed” → “what is dirty” → “what ranges are replayable”

Compatibility strategy:

- Keep `crates/fret-ui` mechanism-only.
- Keep ecosystem policy/recipes stable by providing adapters/shims for a transition period.

### 9.4 Corridor B — “Authoring model v2”: first-class View/Entity composition (GPUI-like)

This corridor is about authoring ergonomics and caching boundaries becoming a primary unit of composition.

1) Introduce an explicit “view entity” authoring layer (ecosystem or new crate), providing:
   - view identity, caching, and dependency observation as a cohesive unit
2) Provide a migration path from today’s `ElementContext` composition:
   - Existing `AnyElement` trees remain valid; views become a wrapper that can host element trees.

Risk/benefit:

- Higher short-term migration cost, but the strongest parity with Zed’s “write UI like Rust” + “cache like views”.

### 9.5 Corridor C — “Identity v2”: debuggable path ids without paying the runtime cost

If/when we decide that `GlobalElementId(u64)` blocks tooling, we can move to a GPUI-like path id model while keeping
runtime performance by splitting identity into:

- a stable opaque key for runtime fast paths, plus
- a debug path representation only when diagnostics are enabled.

Reference:

- GPUI `with_global_id`: `repo-ref/zed/crates/gpui/src/window.rs:2039`

### 9.6 Practical rule: don’t block v2 with v1 decisions

When implementing Phase 0–2 work, enforce the following:

- Any new caching/dispatch APIs must be designed so Phase 3 (prepaint/multi-stream) can be added without breaking semantics.
- Any new “helper” in ecosystem should be a thin layer over stable primitives, not a pile of bespoke runtime hooks.
- Any new “locked contract” should include a short “future v2 note”: what is invariant vs what may change.
