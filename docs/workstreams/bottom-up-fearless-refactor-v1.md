# Bottom-Up Fearless Refactor v1 (Repository-Wide)

Status: Draft (workstream notes only; ADRs remain the source of truth)

Tracking file:

- `docs/workstreams/bottom-up-fearless-refactor-v1-todo.md`

## 0) Why this workstream exists

Fret is intentionally large and demo-driven: it spans platform backends, a UI runtime substrate, a renderer, and an ecosystem of policy-heavy components.
The repository also contains a significant amount of AI-authored code, which makes *consistency* and *long-term maintainability* the primary risk.

This workstream defines a **bottom-up, fearless refactor program**:

- keep shipping demos while refactoring,
- close contracts early (ADRs + tests),
- add executable regressions before expanding surface area,
- and converge code quality toward a “Bevy-like” baseline (clear crates, clear seams, clear ownership).

This document does **not** replace existing subsystem trackers. It provides:

1) a shared refactor loop,
2) cross-cutting guardrails,
3) a milestone taxonomy that can “host” existing workstreams.

## 1) Invariants (do not break)

These are the “hard seams” that make refactors safe:

1. **Mechanism vs policy split is enforced**
   - `crates/fret-ui` remains mechanism-only (ADR 0066 / ADR 0074; see `docs/foundation-first-workflow.md`).
   - shadcn/Radix/APG outcomes remain in ecosystem (`ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`, etc).

2. **Crate layering is enforced**
   - Core crates stay backend-agnostic (no `winit`/`wgpu`/`web-sys` in `fret-core`/`fret-runtime`/`fret-app`/`fret-ui`).
   - Ecosystem crates are extractable and backend-agnostic by default.
   - Validate continuously via `tools/check_layering.py` (ADR 0092, `docs/dependency-policy.md`).

3. **Hard-to-change contracts live in ADRs**
   - If a refactor changes a contract, update or add an ADR first (or explicitly mark a decision gate as proposed).

4. **Demos remain runnable**
   - `fretboard dev native` and at least one “editor-grade” demo stay usable during the program.
   - When behavior changes, add a scripted repro (`fretboard diag`) or a unit/integration test.

5. **No “one big rewrite”**
   - Refactors land in small, behavior-preserving steps, each with a measurable outcome (tests, perf gates, or deletion of unstable surface).

## 1.1) Async policy (do not leak across boundaries)

Async is a major source of accidental coupling and non-determinism in UI frameworks. For this program:

- Do **not** force a global async runtime from core crates.
  - `crates/fret-core`, `crates/fret-runtime`, `crates/fret-ui` must remain usable without Tokio (or any specific executor).
- Keep async work **app-owned** and communicate via explicit messages/effects.
  - Prefer a “Models + Commands + Effects” loop with runner-owned scheduling/draining at the boundary.
- Treat blocking calls on the UI thread as correctness bugs.
  - If a refactor introduces blocking I/O or long CPU work in a hot path, it must be moved behind app-owned async work or a dedicated worker.
- Model backpressure explicitly.
  - “fire-and-forget” async tasks that can grow unbounded should be considered a bug (leaks memory, destroys interactivity).
- Web and native are allowed to differ in executor implementation, but not in the **contract surface**.
  - wasm should use wasm-friendly futures plumbing; native can use Tokio, but the contract must remain portable.

References (guidance, not contracts):

- `docs/integrating-tokio-and-reqwest.md`
- `docs/integrating-sqlite-and-sqlx.md`

## 2) Reference posture (what we borrow, not what we copy)

Local references under `repo-ref/` are non-normative, but they anchor discussions:

- **Bevy** (`repo-ref/bevy`): code quality posture, crate boundaries, error handling, and readability.
- **Zed/GPUI** (`repo-ref/zed`): declarative element tree + retained substrate, caching, identity, and “feel”.
- **shadcn/ui v4** (`repo-ref/ui`) + **Radix Primitives** (`repo-ref/primitives`): component taxonomy and interaction outcomes.

See also: `docs/repo-ref.md`.

## 3) The refactor loop (default workflow)

Use the same loop for any subsystem (core/ui/render/ecosystem):

1. **Pick a closure target**
   - A module/crate boundary, or a hard-to-change behavior that currently “drifts” under changes.

2. **Write down the contract**
   - Link the ADR(s) and add a short “current vs target” list (2–6 bullets).
   - If no ADR exists and the behavior is hard-to-change, create a decision gate first.

3. **Add a regression gate**
   - Prefer unit/integration tests for contracts.
   - Prefer scripted diagnostics (`fretboard diag`) for interaction/perf drift that is hard to unit test.

4. **Refactor in small steps**
   - Each step should either:
     - delete duplication,
     - move code to the correct layer,
     - narrow public surface,
     - or improve correctness/perf with measurable gates.

5. **Audit portability**
   - Replace ad-hoc platform branches with explicit capability signals (see platform capability ADRs/workstreams).

6. **Update alignment tracking**
   - If an ADR is implemented/refactored, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` as required by the repo guidelines.

## 3.1) Module layout policy (crate-internal hygiene)

This workstream is explicitly bottom-up. That only works if crate-internal module boundaries are also treated as
first-class seams (not just workspace crate boundaries).

Principles:

- Prefer **directory modules** for subsystems (`foo/mod.rs` + `foo/*.rs`) once the subsystem exceeds trivial size.
- Prefer grouping related code under a subsystem module rather than using crate-root filename prefixes
  (e.g. prefer `text/{edit,props,surface}.rs` over `text_edit.rs`, `text_props.rs`, `text_surface.rs`).
- Keep “god files” out of the core: large subsystems should be decomposed by responsibility
  (e.g. `dispatch`, `layout`, `paint`, `semantics`, `tests`).
- Keep test code data-driven where possible:
  - large conformance matrices should be represented as external data (`goldens/*.json`) plus a thin harness.

Suggested tactical thresholds (non-normative; use judgement):

- A single `*.rs` file over ~1,000 lines is usually a refactor target.
- A module over ~3,000 lines should almost always be split into submodules with explicit ownership boundaries.

## 4) Milestone taxonomy (program-level)

These milestones are intentionally broad; they are “containers” for existing workstreams and refactor tasks.

### M0 — Guardrails first (make refactors safe)

Outcome: breaking changes become obvious quickly.

- Layering checks are always green and run frequently.
- A small, stable set of smoke tests exists (`cargo nextest run` subsets).
- Diagnostics bundles and scripted repros exist for the top interaction surfaces.

Exit criteria (v1):

- `python3 tools/check_layering.py` is green.
- A documented “refactor safety set” exists (commands + minimal subsets).
- At least one scripted diagnostics suite is considered “always-run” for regressions.
- A lightweight module-size drift report exists (to keep “god files” visible).
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`

### M1 — Core contracts closure (portable kernel confidence)

Outcome: `fret-core`/`fret-runtime`/`fret-app` read like a small kernel with clear ownership.

- Core types are coherent (geometry, ids, input events, services).
- Capability modeling replaces platform forks where possible.
- Public surfaces are minimal and documented.

Exit criteria (v1):

- Each core crate has a module ownership map and a reviewed public export surface.
- No new backend coupling has leaked into core crates (layering remains green).
- At least one regression gate exists for the “core hazard” categories (IDs/stability/serialization invariants).

#### Serialization stability checklist (v1)

Core crates own several “hard-to-change” persisted/config surfaces. For fearless refactors, treat each
surface as a **versioned contract** with at least one executable regression gate.

1) Classify the surface:

- **User-authored config** (e.g. `keymap.json`, menubar config): prefer strict decoding and clear errors.
- **App-owned persisted state** (e.g. docking layouts, window placement, settings snapshots): require an explicit version and a defined compatibility policy.

2) Minimum checklist per surface:

- A version field exists (e.g. `*_version`, `layout_version`) and is checked on load.
- Defaulting is explicit (`#[serde(default)]`) for optional additions that should not break older files.
- Renames are explicit (`#[serde(rename = ...)]`) to avoid accidental drift.
- Validation is explicit and failure messages are actionable (reject invalid graphs/ranges/non-finite values).
- At least one gate exists:
  - **Decode fixture test** (hand-authored JSON samples) for config surfaces.
  - **JSON roundtrip + validate** for app-owned persisted structs (to catch schema drift during refactors).

Evidence anchors (initial):

- Dock layout JSON roundtrip gate: `crates/fret-core/src/dock/tests.rs` (`dock_layout_json_roundtrips_and_validates`)

### M2 — UI runtime closure (mechanism-only, debuggable, cache-safe)

Outcome: `crates/fret-ui` is a stable substrate, not a component library.

- Layout engine direction is clear and regression-tested (v2 roots, barriers, invariants).
- Overlay + input arbitration is deterministic and tested (pointer occlusion, lifecycle phases).
- Element identity + state + invalidation story is explainable via tooling.

Exit criteria (v1):

- Mechanism/policy boundary is continuously enforced (no “outcome policy defaults” in `crates/fret-ui`).
- A “refactor hazards” list exists for `crates/fret-ui`, and each hazard has at least one gate.
- Module layout hygiene improvements land without widening the public contract surface.

#### Fret UI refactor hazards (v1)

This list is meant to keep `crates/fret-ui` refactors fearless: each hazard should have at least one
executable gate (unit/integration/diag) that fails when behavior drifts.

1. **Layout recursion / stack safety**
   - Failure mode: infinite layout loops, unbounded recursion, stack overflows.
   - Gates:
     - `crates/fret-ui/src/tree/tests/stack_safety.rs`
     - `crates/fret-ui/src/declarative/tests/layout.rs`

2. **Overlay dismissal + outside-press semantics**
   - Failure mode: click-through, missed outside-press, escape dismissal drift, focus-loss surprises.
   - Gates:
     - `crates/fret-ui/src/tree/tests/outside_press.rs`
     - `crates/fret-ui/src/tree/tests/escape_dismiss.rs`

3. **Pointer occlusion + capture arbitration**
   - Failure mode: wrong recipient gets pointer events; occlusion layers regress; capture leaks.
   - Gates:
     - `crates/fret-ui/src/tree/tests/pointer_occlusion.rs`
     - `crates/fret-ui/src/tree/tests/window_input_arbitration_snapshot.rs`

4. **Hit testing correctness + cache reuse policy**
   - Failure mode: stale hit paths, incorrect reuse of hit-test caches, “ghost” interactions.
   - Gates:
     - `crates/fret-ui/src/tree/tests/hit_test.rs`
     - `crates/fret-ui/src/tree/tests/hit_test_cache_reuse_policy.rs`

5. **Focus scopes + traversal availability**
   - Failure mode: focus trap/restore drift, traversal gating mismatches, unexpected focus loss.
   - Gates:
     - `crates/fret-ui/src/tree/tests/focus_scope.rs`
     - `crates/fret-ui/src/tree/tests/focus_traversal_availability.rs`
     - `crates/fret-ui/src/tree/tests/window_command_action_availability_snapshot.rs`

6. **Text input + IME snapshot correctness**
   - Failure mode: preedit ranges drift, composition state leaks, snapshot contract breaks.
   - Gates:
     - `crates/fret-ui/src/tree/tests/platform_text_input.rs`
     - `crates/fret-ui/src/tree/tests/window_text_input_snapshot.rs`

7. **Scroll + virtual list invalidation classification**
   - Failure mode: scroll changes force full relayout/paint unexpectedly, overscan drift, jank.
   - Gates:
     - `crates/fret-ui/src/tree/tests/scroll_invalidation.rs`
     - `crates/fret-ui/src/tree/tests/scroll_into_view.rs`
     - `crates/fret-ui/src/declarative/tests/virtual_list.rs`

8. **View cache / paint cache correctness**
   - Failure mode: cached subtrees fail to invalidate or invalidate too broadly.
   - Gates:
     - `crates/fret-ui/src/tree/tests/view_cache.rs`
     - `crates/fret-ui/src/tree/tests/paint_cache.rs`
     - `crates/fret-ui/src/declarative/tests/view_cache.rs`

9. **Dispatch phases + command availability snapshots**
   - Failure mode: input routing order changes, command gating drifts, post-dispatch snapshots regress.
   - Gates:
     - `crates/fret-ui/src/tree/tests/dispatch_phase.rs`
     - `crates/fret-ui/src/tree/tests/window_input_context_snapshot.rs`

10. **Cross-frame identity + GC liveness**
    - Failure mode: element identity becomes unstable; state leaks or is dropped unexpectedly.
    - Gates:
      - `crates/fret-ui/src/declarative/tests/identity.rs`
      - `crates/fret-ui/src/declarative/tests/element_state_gc.rs`
      - `crates/fret-ui/src/tree/tests/gc_liveness.rs`

### M3 — Renderer closure (GPU-first, predictable, inspectable)

Outcome: rendering + text pipelines are refactorable without “mystery regressions”.

- GPU context hosting story is locked (ADR 0010 and related renderer ADRs).
- Text/shaping/atlas caches have observable performance snapshots.
- RenderDoc/Tracy workflows are documented and used for regressions.

Exit criteria (v1):

- At least one renderer-heavy stress harness is used as a regression gate (perf snapshot checks or diag bundle checks).
- Profiling/inspection workflows are runnable on Windows and produce reproducible artifacts.

#### Renderer regression surfaces (v1)

This list is intentionally small: it defines the “if this drifts, we want a fast failure” surfaces
for renderer refactors.

1. **Shader compilation + WebGPU validation**
   - Gates:
     - `crates/fret-render/src/renderer/mod.rs`

2. **RenderPlan compilation (effect graph encoding)**
   - Failure mode: pass ordering/scissor mapping/clip mask budgeting changes subtly.
   - Gates:
     - `crates/fret-render/src/renderer/render_plan.rs`

3. **Intermediate target pool + budgets**
   - Failure mode: uncontrolled target growth or eviction drift causing perf/memory regressions.
   - Gates:
     - `crates/fret-render/src/renderer/intermediate_pool.rs`

4. **Clip/scissor correctness**
   - Failure mode: clipped effects bleed outside bounds, scissor mapping off-by-ones.
   - Gates:
     - `crates/fret-render/tests/affine_clip_conformance.rs`
     - `crates/fret-render/tests/postprocess_scissor_conformance.rs`

5. **Backdrop effects correctness (blur/pixelate/color adjust)**
   - Failure mode: wrong anchoring, wrong ordering, wrong clip handling.
   - Gates:
     - `crates/fret-render/tests/effect_backdrop_blur_conformance.rs`
     - `crates/fret-render/tests/effect_backdrop_pixelate_conformance.rs`
     - `crates/fret-render/tests/effect_backdrop_color_adjust_conformance.rs`

6. **Filter-content effects correctness**
   - Gates:
     - `crates/fret-render/tests/effect_filter_content_blur_conformance.rs`
     - `crates/fret-render/tests/effect_filter_content_pixelate_conformance.rs`

7. **Text shaping/wrapping + key stability**
   - Failure mode: shaping/paint cache keys drift, wrapping semantics regress.
   - Gates:
     - `crates/fret-render/src/text/mod.rs`
     - `crates/fret-render/src/text/wrapper.rs`
     - `crates/fret-render/src/text/parley_shaper.rs`

8. **SVG rasterization outputs**
   - Failure mode: alpha masks/rgba renders change unexpectedly, breaking icon rendering.
   - Gates:
     - `crates/fret-render/src/svg/mod.rs`

### M4 — Ecosystem rationalization (policy surfaces scale safely)

Outcome: ecosystem crates align to a coherent layering: headless → kit → shadcn → specialized kits.

- No policy leaks back into `crates/*`.
- Retained bridge blast radius shrinks (or becomes trivially removable).
- Authoring ergonomics converge (fluent builder, consistent `test_id` conventions, stable recipes).

Exit criteria (v1):

- Any usage of `fret-ui/unstable-retained-bridge` is tracked (allowlist + migration plan) and trending downward.
- Ecosystem crates are backend-agnostic by default (layering remains green).

### M5 — “Open-source readiness” (optional, future)

Outcome: if/when we open-source, the repo is understandable and dependable.

- Docs entrypoints are accurate and current.
- CI gates cover the critical invariants (layering, formatting, clippy, smoke tests).
- The “golden path” demos and templates are reproducible.

## 4.1) Crate-by-crate sequence (bottom-up)

The default execution order for this program is:

1. `crates/fret-core` (types, IDs, geometry, portable event model)
2. `crates/fret-runtime` (portable services and value types; effects boundary)
3. `crates/fret-app` (models/commands/effects plumbing; scheduling; app-owned side effects)
4. `crates/fret-ui` (runtime substrate; mechanism-only; layout/dispatch/paint/semantics/overlays)
5. `crates/fret-render` (renderer internals; RenderPlan substrate; text rendering performance)
6. `crates/fret-platform*`, `crates/fret-runner-*`, `crates/fret-launch` (backend adaptation and glue)
7. `ecosystem/*` (policy surfaces; authoring ergonomics; shadcn/radix alignment; specialized kits)
8. `apps/*` (demo shells; UI gallery; diag harnesses)

For each crate, do not “move on” until:

- module layout is coherent (no ambiguous `foo.rs`/`foo/` splits),
- public surface is intentional (exports reviewed; no accidental re-exports),
- at least one regression gate exists for the crate’s refactor hazards,
- and layering checks remain green.

Rationale:

- `fret-core`/`fret-runtime` establish the vocabulary and guard against “dependency creep”.
- `fret-ui` is the highest churn hotspot; it benefits the most from having stable, clean lower layers first.
- apps are intentionally last: they should consume stabilized contracts, not define them.

## 4.2) Per-crate closure checklist (template)

Use this checklist when refactoring a crate bottom-up:

1. **Module map**
   - Write a short “module ownership map” (subsystems, their responsibilities, and the primary files).

2. **Public surface audit**
   - Review `lib.rs` exports and re-exports.
   - Remove accidental exports; keep the public surface intentional and documented.

3. **Module layout hygiene**
   - Group subsystems under modules (avoid crate-root prefix files).
   - Keep files small enough to review comfortably; split by responsibility.

4. **Hazards + gates**
   - Identify 3–10 refactor hazards for the crate.
   - Add at least one regression gate per hazard category (unit/integration/diag).

5. **Portability + capability modeling**
   - Replace platform forks with explicit capability signals where applicable.

6. **Docs alignment**
   - Link the work to ADRs and update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` when relevant.

## 4.3) Code-quality audits (crate-by-crate, bottom-up)

The directory/module reshuffles in this workstream are only the first pass. To converge toward a
“Bevy-like” baseline, we will likely need a second pass that is explicitly **code-quality driven**:

- read each crate end-to-end (or at least its critical paths),
- audit for Rust best practices and architectural intent,
- and land small, gated refactors that reduce long-term maintenance risk.

### Audit outputs (lightweight, reviewable artifacts)

- A stable template: `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audit-template.md`
- A single index for progress tracking: `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audits.md`
- One “crate audit note” per crate (linked from the index) once we start doing deep dives.

### Audit levels (so we can scale across a large repo)

- **L0 — Quick scan (30–60 min)**: surface map, top hazards, biggest files, dependency smells.
- **L1 — Targeted deep dive (half-day)**: critical paths + invariants, remove obvious footguns,
  add at least one executable gate.
- **L2 — Closure audit (multi-day)**: contract closure + portability review + perf/interaction gates,
  plus alignment updates if ADR-covered behavior is touched.

### Minimum gates per audit level (recommended)

These are intentionally small “default minima” so audits remain landable.
If a change touches a hot path (dispatch/layout/paint) or interaction semantics, add additional gates.

| level | minimum gates | typical outputs |
| --- | --- | --- |
| L0 | `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate <crate>` | 3–10 hazards + a short next-steps list |
| L1 | L0 + `pwsh -NoProfile -File tools/check_layering.ps1` + `cargo fmt` + `cargo nextest run -p <crate>` | at least one new regression gate, plus 3–8 landable refactor steps |
| L2 | L1 + `cargo clippy --workspace --all-targets -- -D warnings` + at least one `fretboard diag` suite or perf gate (as applicable) | contract closure notes, portability review, and ADR alignment updates if touched |

### What “Rust best practices” means in this repo (non-normative)

We prioritize changes that reduce risk and improve reviewability over micro-optimizations.
Common targets to look for during audits:

- **API surface hygiene**
  - Keep public exports intentional; avoid “accidental” re-exports.
  - Prefer data-only contract types in portable crates; keep platform bindings behind adapters.
- **Ownership clarity**
  - Avoid “god modules” and unclear ownership; split by responsibility and keep facades thin.
  - Prefer directory modules once a subsystem is non-trivial.
- **Error handling**
  - Use structured error types (`thiserror`) for contract surfaces; reserve `anyhow`-style context for
    app/backends where the contract is “best effort”.
  - Avoid `unwrap()`/`expect()` in production paths (tests are fine).
- **Determinism and testability**
  - Prefer deterministic iteration and explicit ordering in user-visible behavior.
  - Extract pure helpers where possible and add unit tests around invariants.
- **Hot-path discipline**
  - Avoid accidental allocations/clones in dispatch/layout/paint hot paths.
  - Avoid blocking I/O or long CPU work on the UI thread; push it behind effects/dispatchers.
- **Unsafe and platform glue**
  - Keep `unsafe` localized, documented, and covered by tests where possible.
  - Prefer capability modeling over ad-hoc `cfg` forks in portable layers.

If a refactor changes an ADR-covered contract, it must go through ADR alignment workflow (see repo
guidelines in `docs/README.md`).

## 5) Where this workstream plugs in (existing trackers)

This program should not duplicate detailed plans; it should link and provide cross-cutting guardrails.
Examples:

- M0/M2: `docs/workstreams/foundation-closure-p0.md`
- M2: `docs/layout-engine-refactor-roadmap.md`, `docs/workstreams/overlay-input-arbitration-v2.md`,
  `docs/workstreams/gpui-parity-refactor.md`, `docs/workstreams/retained-bridge-exit-v1.md`
- M3: `docs/renderer-refactor-roadmap.md`, `docs/tracy.md`, `docs/renderdoc-inspection.md`
- M4: `docs/shadcn-declarative-progress.md`, `docs/workstreams/ecosystem-status.md`

## 6) Working agreements (practical rules)

- Prefer renames/moves over new abstractions unless there is a proven reuse target.
- Avoid widening public APIs while refactoring; shrink first, then grow behind ADR gates.
- If a refactor touches a hot path, land a perf gate (even a crude one) before optimizing.
- If a refactor touches UI interaction semantics, land at least one scripted repro.

## 6.1) “Always-run” gates (recommended defaults)

This program is only “fearless” if we have cheap, repeatable gates.
Recommended default gates (adjust per workstream):

- Layering: `python3 tools/check_layering.py`
- Format: `cargo fmt`
- Lint (when affordable): `cargo clippy --workspace --all-targets -- -D warnings`
- Tests (subset, then expand): `cargo nextest run -p fret-ui` and `cargo nextest run -p fret-ui-shadcn`
- Diag (interaction subset): `cargo run -p fretboard -- diag suite ui-gallery-overlay-steady --env FRET_DIAG=1 --launch -- cargo run -p fret-ui-gallery --release`

Gate tiers (suggested; tune to your machine/CI budgets):

- Fast (developer inner loop): layering + fmt + a small nextest subset for the touched crate(s).
- Full (pre-merge / nightly): layering + fmt + clippy + wider nextest coverage + at least one diag suite.

Canonical scripts (keep these stable so “fearless” stays repeatable):

- Fast: `pwsh -NoProfile -File tools/gates_fast.ps1`
- Full: `pwsh -NoProfile -File tools/gates_full.ps1`
  - Note: the heaviest shadcn web-golden-backed conformance tests are gated behind
    `--features web-goldens` for `fret-ui-shadcn` to keep the default inner loop cheaper.

## 6.2) Skills (recommended)

Repo-local skills exist to make the refactor loop repeatable for humans and AI.
When doing work under this workstream, prefer using the following skills as “procedural checklists”:

- `fret-boundary-checks`: layering + largest-files drift + quick crate snapshots.
- `fret-crate-audits`: L0/L1/L2 audit workflow and artifact expectations.
- `fret-diag-workflow`: turn regressions into reproducible `fretboard diag` gates.
- `fret-app-architecture-and-effects`: async/effects boundaries; avoid UI-thread blocking.
- UI-focused skills as needed (`fret-overlays-and-focus`, `fret-text-input-and-ime`, `fret-layout-and-style`, `fret-scroll-and-virtualization`).

## 7) Initial “mis-modularization” targets (concrete examples)

These are examples of the type of crate-internal module hygiene issues this workstream should fix early:

- `crates/fret-ui` text-related code is split across multiple roots (`text_input/`, `text_area/`, plus `text_*` files at crate root).
  Prefer regrouping these into a single `text/` subsystem module with clear submodules (`input`, `area`, `edit`, `props`, `style`, `surface`),
  keeping public exports unchanged.
- Large conformance tests should avoid living as multi-ten-thousand-line Rust sources; prefer `goldens/` + a thin harness.
  - Early targets:
    - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
    - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
- `ecosystem/fret-ui-kit` overlay tests are currently concentrated in a very large `src/window_overlays/tests.rs`.
  Prefer splitting by scenario class + moving scenario matrices to fixtures to reduce merge/conflict risk and improve reviewability.
