# ADR 0190: Execution and Concurrency Surface v1 (Dispatcher + Executors)

Status: Accepted

## Context

Fret targets both:

- small apps that should feel straightforward to author, and
- editor-grade apps (multi-window, docking, heavy models, background services).

We already have a hard constraint that keeps the architecture predictable:

- the platform event loop, UI updates, scene building, and rendering are driven from the main thread,
- background work communicates back via data-only messages (`docs/adr/0008-threading-logging-errors.md`).

We also document recommended patterns for async/background work in the golden path driver
(`docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`), but today those patterns are mostly
guidance, not a shared, reusable surface. This leads to:

- repeated bespoke `mpsc`/channels + timers + wake logic in apps and ecosystem crates,
- inconsistent cancellation / wake semantics,
- portability traps (native vs wasm) and runner-coupling (accidentally depending on `winit`/`wgpu`),
- harder debugging and observability because "how the UI got woken" varies per project.

We want a small, stable execution surface that:

- preserves the "main-thread mutation" invariant,
- does not force a specific async runtime (Tokio/async-std),
- works across native and wasm,
- offers good ergonomics for both end users and third-party ecosystem authors,
- leaves a clear migration path to more sophisticated threading/lane layouts if needed.

## Goals

- Provide a stable, portable contract for:
  - scheduling work onto the main thread,
  - running background work,
  - timers/delays,
  - waking the UI loop promptly from background completion.
- Make "background → data-only message → main-thread apply → request redraw" the default story.
- Keep ecosystem crates portable (no backend deps) while still enabling background work.
- Preserve a clear path to future evolution (lane splitting, priorities, backpressure) without
  breaking user code.

## Non-goals

- Forcing a particular async runtime.
- Mandating a separate render thread or defining a renderer threading model.
- Making `crates/*` depend on `ecosystem/*`.
- Hiding the Event / Command / Effect pipeline model (ADR 0112).

## Decision

### 1) Keep the main-thread mutation invariant as a hard contract

- `App` / `ModelStore` (and any mutable UI/runtime state) are main-thread only.
- Background tasks MUST NOT mutate `App` directly.
- Background work MUST communicate results via data-only messages that are applied on the main
  thread at an explicit, runner-owned boundary (tick/flush point), followed by redraw scheduling.

This contract remains the primary tool for avoiding pervasive locking and keeping behavior
deterministic in multi-window applications.

### 2) Introduce a portable `Dispatcher` contract (runner-provided)

We standardize a minimal interface, owned by a portable crate (recommended placement:
`crates/fret-runtime`) and implemented by platform runners (e.g. `fret-launch` for desktop/winit,
web runner for wasm).

The contract is semantic, not implementation-specific:

- **Dispatch to main thread**: schedule a runnable to execute on the UI/main thread.
- **Dispatch to background**: schedule a runnable to execute off the UI thread (or "not on the UI
  queue" in constrained environments).
- **Dispatch after**: schedule delayed work (timers).
- **Wake**: request that the runner reaches the next safe driver boundary promptly (end of event
  tick / effect drain / frame boundary), so queued work can be observed and applied.

Important invariants:

- The mapping between the `Dispatcher` and the underlying servicing threads/queues MUST remain
  stable for the lifetime of the process (no dynamic re-binding).
- `dispatch_on_main_thread` MUST preserve thread affinity (the runnable executes on the UI/main
  thread, even if the platform internally uses multiple callbacks/queues).
- On wasm, "background" may be best-effort (no threads); the API remains, but implementations may
  execute work cooperatively and MUST still preserve the main-thread mutation invariant.

#### Portability and degraded guarantees (required guidance)

The runner MUST surface execution capabilities via `PlatformCapabilities.exec` (ADR 0054). Ecosystem
crates SHOULD avoid forking on capabilities unless absolutely necessary, but these keys exist to make
degradation explicit and diagnosable:

- Native (typical): `exec.background_work=threads`, `exec.wake=reliable`, `exec.timers=reliable`
- wasm (typical): `exec.background_work=cooperative`, `exec.wake=best_effort`, `exec.timers=best_effort`

Implications for authors:

- Do not assume background work implies a separate OS thread. On wasm without threads, "background"
  work may run cooperatively on the same thread and can still block the UI if it is CPU-heavy.
- Treat `wake()` as a request to reach the next driver boundary, not a precise scheduling guarantee.
  On wasm it may be delayed or coalesced.
- Prefer the runner-owned effect pipeline for UI-visible timing (`Effect::SetTimer` / RAF). Treat
  `dispatch_after` as a low-level primitive for executors/harnesses, and expect timer throttling on
  constrained platforms.

#### Placement and stability rules

To preserve long-term flexibility while still enabling third-party ecosystems:

- The **minimal, semantic** `Dispatcher` trait belongs in a portable framework crate (recommended:
  `crates/fret-runtime`) so ecosystem crates can depend on it without pulling in backends.
- Higher-level ergonomics (executors, inbox helpers, cancellation utilities) MUST live in
  `ecosystem/*` so they can iterate without forcing kernel churn.
- Any "advanced" features (priorities beyond a small enum, queue isolation, backpressure, task
  attribution hooks) SHOULD be added via extension traits or optional adapter crates, not by
  expanding the base trait prematurely.

#### Driver boundaries and `wake()` semantics

We standardize terminology and guarantees so implementations remain consistent:

- **Driver boundary**: the runner-owned point where it is safe to:
  - drain background inboxes,
  - apply model/global updates,
  - drain effects,
  - schedule redraw and (if needed) begin a new frame.

Examples of valid driver boundaries:

- end of an event-loop tick after input events have been routed,
- end of an effect drain pass,
- start of the next frame build.

`wake()` semantics:

- `wake()` requests that the runner reaches the next driver boundary promptly.
- `wake()` MAY be coalesced (multiple calls collapse into one wake).
- `wake()` MUST NOT be interpreted as "run tasks immediately" or "force a frame"; it is a prompt
  to advance the driver to its next safe boundary.
- `wake()` SHOULD be **window-scoped** where the runner supports multiple windows (wake the driver
  boundary for the affected window). If a runner only supports a global wake primitive, it MUST
  document that wake is global and MAY result in unrelated windows reaching a driver boundary.

This keeps wake behavior portable across:

- desktop (event loop proxy),
- wasm (RAF/microtask/timeout scheduling),
- mobile (Choreographer / CADisplayLink / OS main-thread run loop).

#### Driver boundary ordering (v1 baseline)

To reduce cross-crate ambiguity, we define a baseline ordering that runners and the golden path
should preserve:

1. Collect platform events for a tick (or the next frame boundary).
2. Route input events through the UI runtime (event pipeline).
3. Drain background inboxes (data-only messages).
4. Apply model/global updates on the main thread.
5. Drain effects at a flush point (effect pipeline).
6. Schedule redraw / begin frame work as needed.

Runners MAY split these into finer phases, but the relative ordering above SHOULD remain stable so
"background completion → wake → next boundary → UI sees the update" is predictable.

### 3) Standardize two executor roles as ecosystem-level ergonomics (portable)

To make the above contract easy to use, we standardize two executor roles and recommend providing
them as ecosystem-level convenience surfaces (e.g. `ecosystem/fret-bootstrap` or a dedicated
`ecosystem/fret-executor` crate):

- **ForegroundExecutor**:
  - schedules and polls tasks on the UI thread,
  - MAY support `!Send` futures/tasks,
  - intended for "continue on the UI thread" work.
- **BackgroundExecutor**:
  - schedules work off the UI thread where available,
  - accepts `Send` tasks by default,
  - supports cancellation as the default behavior.

Cancellation baseline:

- Dropping a task handle SHOULD cancel the work (or at minimum guarantee that completion does not
  attempt to call back into the UI after shutdown/hot reload).
- A `detach`-style API MAY exist for intentional fire-and-forget work.

#### Cancellation and drop safety

To avoid subtle cross-thread drop hazards (especially with `!Send` futures):

- Foreground task handles SHOULD be `!Send` so they cannot be moved/dropped from non-main threads
  in safe Rust.
- Background tasks MUST ensure cancellation is thread-safe and that "cancelled" tasks do not
  enqueue UI work after the app/runner has begun shutting down.
- If the runner is shutting down and a foreground runnable cannot be delivered to the main thread,
  implementations MAY choose to "forget" the runnable rather than dropping it on the wrong thread.
  (Safety is preferred over perfect cancellation in shutdown paths.)

#### Cancellation minimum guarantees (required)

Once cancellation begins (task dropped/cancelled, runner shutdown, hot reload reset):

- background tasks MUST NOT enqueue new UI/main-thread work,
- background tasks MUST NOT call `wake()` for UI work,
- inbox sends SHOULD either be rejected or safely dropped (and counted) if the consumer is gone.

This prevents "ghost callbacks" and makes shutdown/hot reload behavior deterministic.

### 4) Make "Inbox + wake + flush point" the canonical integration pattern

We standardize a small, portable pattern that ecosystem crates can rely on:

- background producers send data-only messages into an inbox,
- the UI/main thread drains the inbox at an explicit driver boundary,
- draining applies updates and schedules redraw.

This pattern is the portable default because it:

- works on native and wasm,
- keeps the "only main mutates state" invariant,
- composes cleanly with the Event / Command / Effect pipelines (ADR 0112),
- is friendly to hot reload reset boundaries (ADR 0107).

#### Inbox backpressure (required)

To avoid unbounded memory growth and "drain stalls" in heavy apps:

- Inbox implementations SHOULD be bounded, or support a configurable bound.
- When the bound is exceeded, implementations MUST apply a documented strategy, for example:
  - drop-oldest,
  - drop-newest,
  - coalesce-by-key (latest-wins per key).
- Runners/diagnostics SHOULD expose counters for dropped/coalesced messages so issues are visible.

### 5) Timers: one scheduling substrate, two entry points

We must avoid "split brain" timer stacks.

- **User-facing/UI-affecting timers** SHOULD flow through effects (e.g. `Effect::SetTimer`) so they
  remain visible in the Effect pipeline model (ADR 0112) and are owned by the runner.
- The `Dispatcher` MAY still expose a low-level `dispatch_after` primitive for implementing
  executor utilities (e.g. background timeouts, cooperative scheduling, test harnesses), but it
  MUST share the same underlying scheduling substrate/time base as effect timers.

### 6) Lane model (future-proofing without committing to a split today)

We define a conceptual "lane" model that can evolve without breaking the above contract:

- **Main/UI lane** (required): owns `App` mutation and UI tree work.
- **Platform lane** (optional): OS integration callbacks; may be the same as Main/UI.
- **Render lane** (optional): GPU submission/present; may be the same as Main/UI.
- **IO lane** (optional): expensive decoding/upload work; may be the same as Background.

v1 does not require these lanes to exist as separate threads. The only requirement is that the
`Dispatcher` and executors preserve the main-thread mutation contract and provide a wake mechanism.

If future platforms/perf constraints require lane splitting, we can:

- extend `Dispatcher` to express lane-targeted scheduling,
- update runners to map lanes to threads,
- keep existing user code valid (still only mutates on Main/UI and communicates via messages).

## Mobile Mapping (Non-normative)

This section is guidance and does not add new hard contracts beyond the Decision above.

### iOS

- `dispatch_on_main_thread` maps to the OS main thread (UIKit-affine).
- `wake()` maps to a "reach next driver boundary" request via the platform run loop / display link.
- Background execution exists but may be constrained; cancellation and "no UI callbacks after
  shutdown" are essential.
- Surface/scene lifecycles must tolerate frequent suspend/resume and "surface lost" style events;
  runners SHOULD cancel or fence in-flight work during teardown boundaries.

### Android

- `dispatch_on_main_thread` maps to the Android main thread (Looper-affine).
- `wake()` maps to a prompt via Choreographer or posting to the main Looper to ensure the next
  driver boundary is reached promptly.
- Background execution exists but must avoid blocking the main thread (ANR/watchdogs); long work
  belongs on BackgroundExecutor and returns via inbox messages.

## Ecosystem Author Experience (Third-Party Libraries)

Third-party ecosystem crates (plots, node graphs, markdown, docking policy, LSP UI integration) should:

- depend only on portable surfaces (`fret-core` / `fret-runtime` / `fret-ui` / ecosystem helpers),
  not backend crates (`winit`, `wgpu`, `web-sys`),
- use the standardized inbox + dispatcher/executor surfaces to run background work,
- keep their own "engine/headless tier" (optional) purely synchronous/pure-data when possible,
  so it can be tested deterministically and used in wasm constraints.

Recommended integration shape for ecosystem crates:

- **Headless tier** (optional): pure models/algorithms; no async runtime dependencies.
- **UI integration tier**: uses `Dispatcher` (or higher-level executors) to schedule background work
  and returns results via inbox messages applied on the UI thread.

This keeps third-party crates portable while still enabling "real app" workloads.

## Consequences

### Benefits

- A single, reusable execution surface improves ergonomics and reduces bespoke concurrency code.
- Clear portability story across native and wasm without forcing a runtime.
- Easier observability: runners can instrument dispatch/wake points consistently via tracing spans.
- Enables larger applications to adopt richer execution models (Tokio, thread pools, priorities)
  via adapters, without changing core UI contracts.

### Costs

- Requires careful API shaping to avoid locking in the wrong abstractions too early.
- Introduces a new "official" surface that must be documented and kept stable once accepted.

## Alternatives Considered

- **Keep it as guidance only** (status quo): rejected because it perpetuates duplicated wake/cancel
  logic and inconsistent portability.
- **Force Tokio**: rejected (conflicts with existing policy and wasm constraints).
- **Define explicit TaskRunners (platform/ui/render/io) now**: deferred; this is higher complexity
  and can be layered later via the lane model without breaking v1.

## Migration Plan

1. Land this ADR as `Accepted` once the surface is deemed stable.
2. Update the golden path driver to rely on the standardized dispatcher/executor/inbox pattern.
3. Provide templates/examples that use the surface by default (small apps) and show an adapter path
   for heavy apps (Tokio thread + wake).
4. Gradually update ecosystem crates to converge on the shared surface (reducing bespoke channels).

## Acceptance Criteria (Satisfied)

This ADR may transition to `Accepted` once all of the following are true:

- **Two runner mappings exist** (at minimum):
  - one native runner (desktop) implementation of `Dispatcher` with a real `wake()` mechanism,
  - one wasm/web implementation with defined "background best-effort" behavior and `wake()` mapping.
- **Golden path uses the surface end-to-end**:
  - a reference app (e.g. `Todo`-class) uses inbox + wake + driver boundary draining,
  - no bespoke channels/timers are required in the user-facing "starter" code.
- **Third-party author story is validated**:
  - at least one ecosystem crate adopts the inbox/dispatcher pattern for background work without
    depending on backend crates (`winit`/`wgpu`/`web-sys`).
- **Testing story exists**:
  - deterministic tests cover wake/drain ordering (a test dispatcher or equivalent harness),
  - shutdown/cancellation paths are tested (no UI callbacks after shutdown).
- **Observability hooks exist**:
  - dispatch/wake/drain points are visible in tracing spans (location attribution where feasible).

Evidence (as of 2026-01-26):

- `Dispatcher` + `InboxDrainRegistry`: `crates/fret-runtime/src/execution.rs`
- Desktop/wasm runner mappings + driver boundary draining: `crates/fret-launch/src/runner/{desktop,web}/dispatcher.rs`, `crates/fret-launch/src/runner/{desktop/mod.rs,web.rs}`
- Ecosystem + examples + tests: `ecosystem/fret-executor/src/lib.rs`, `ecosystem/fret-markdown/src/mathjax_svg_support.rs`, `apps/fret-examples/src/markdown_demo.rs`

## References

- Threading policy and error boundaries: `docs/adr/0008-threading-logging-errors.md`
- Timers and scheduling: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Golden path pipelines and async patterns: `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`
- Ecosystem integration guidance: `docs/adr/0113-ecosystem-integration-contracts.md`
- Dev hotpatch reset boundaries: `docs/adr/0107-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Platform capabilities matrix (portability diagnostics): `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Implementation plan (notes): `docs/workstreams/execution-concurrency-surface-v1.md`
- Non-normative references:
  - Flutter TaskRunner model (embedder-configurable runners): `repo-ref/flutter/docs/about/The-Engine-architecture.md`
  - GPUI dispatcher/executor substrate: `repo-ref/zed/crates/gpui/src/executor.rs`, `repo-ref/zed/crates/gpui/src/platform_scheduler.rs`
