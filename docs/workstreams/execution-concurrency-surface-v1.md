# Execution & Concurrency Surface v1 (Implementation Plan)

This document is a workstream note that expands `docs/adr/0190-execution-and-concurrency-surface-v1.md`
into a concrete implementation/migration plan. It is **not** a stable contract; the ADR is.

## Why this exists

We want a shared execution surface that:

- preserves the "main thread mutates `App`" invariant,
- keeps ecosystem crates portable (no backend deps),
- gives small apps a default, ergonomic story,
- scales to heavy editor apps (Tokio, thread pools, background services),
- remains compatible with wasm and future mobile backends.

The risk we are avoiding is "every ecosystem crate invents its own channels + wake + timers", which
creates portability traps and makes debugging inconsistent.

## Tracking (living TODOs)

Last updated: 2026-01-26

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

### Phase 0 (docs only)

- `[x]` ADR: lock the surface and semantics (`docs/adr/0190-execution-and-concurrency-surface-v1.md`)
- `[x]` Portability keys: add `exec.*` to the capabilities matrix (`docs/adr/0054-platform-capabilities-and-portability-matrix.md`)
- `[x]` Runtime matrix: add an execution/wake/timers portability line (`docs/runtime-contract-matrix.md`)
- `[x]` Docs: update `docs/crate-usage-guide.md` ("Background work" recommended surface)
- `[x]` Docs: update `docs/effects-authoring.md` (timer story: `Effect::SetTimer` vs `dispatch_after`)
- `[x]` Docs: update `docs/ui-ergonomics-and-interop.md` (heavy app adapter: Tokio thread + wake + inbox)

### Phase 1 (desktop runner + golden path)

- `[x]` `Dispatcher` trait: define minimal portable surface (target: `crates/fret-runtime`, implemented at `crates/fret-runtime/src/execution.rs`)
- `[x]` Desktop impl: implement `Dispatcher` in the desktop runner (target: `crates/fret-launch`, implemented at `crates/fret-launch/src/runner/desktop/dispatcher.rs`)
- `[x]` Runner: route platform completions through the dispatcher background lane (target: `crates/fret-launch/src/runner/desktop/mod.rs`)
- `[x]` Ergonomics: add ecosystem executors + inbox helpers (target: `ecosystem/*`, implemented as `ecosystem/fret-executor`)
- `[x]` Driver boundary: drain inboxes at runner flush points via `InboxDrainRegistry` (desktop: `crates/fret-launch/src/runner/desktop/mod.rs`, web: `crates/fret-launch/src/runner/web.rs`)
- `[x]` Observability: add tracing spans for dispatch/wake/drain points (dispatcher + driver boundary)
- `[x]` Safety: fence dispatcher on shutdown (desktop + wasm), plus basic unit tests
- `[x]` Safety: cancellation baseline (ecosystem task handles; no post-cancel enqueue/wake)
- `[x]` Safety: hot reload boundary (desktop runner bumps dispatcher generation and clears pending work)

### Phase 2 (wasm mapping)

- `[x]` wasm impl: define `dispatch_on_main_thread`/`dispatch_after`/`wake` mapping (timeout + runner proxy wake; implementation in `crates/fret-launch/src/runner/web/dispatcher.rs`)
- `[x]` wasm "background": define cooperative/best-effort behavior (no threads; `spawn_local`-based)
- `[x]` Portability docs: explicitly document degraded guarantees and recommended patterns

### Phase 3 (ecosystem validation)

- `[x]` Migrate 1 ecosystem crate to the shared surface (`ecosystem/fret-markdown` MathJax SVG worker)
- `[x]` Add deterministic tests for wake/drain ordering in that crate (no real timers required)

### Phase 4 (acceptance)

- `[x]` Meet ADR 0190 acceptance criteria and flip status to `Accepted`
- `[x]` Replace remaining bespoke channel+wake utilities in templates/examples
- `[x]` Guardrails: add CI check that rejects raw `std::thread::{spawn,sleep}` and bespoke channels/timer crates in ecosystem/apps (`tools/check_execution_surface.ps1`)

## Proposed public surface (API sketch, non-binding)

This is an API sketch to align on shape and ownership. Naming is intentionally non-final.

### Core trait: `Dispatcher` (portable)

Target placement: `crates/fret-runtime`.

Key constraints:

- `dispatch_on_main_thread` MUST execute on the UI/main thread.
- `wake` MUST advance the runner to the next driver boundary (may be coalesced).
- wasm may implement "background" as best-effort (cooperative), but MUST still preserve the main-thread mutation invariant.

Sketch:

- `trait Dispatcher: Send + Sync + 'static`
  - `fn dispatch_on_main_thread(&self, task: Runnable)`
  - `fn dispatch_background(&self, task: Runnable, priority: Priority)`
  - `fn dispatch_after(&self, delay: Duration, task: Runnable)`
  - `fn wake(&self, window: Option<AppWindowId>)`
  - `fn exec_capabilities(&self) -> ExecCapabilities` (integrated into `PlatformCapabilities.exec` via ADR 0054 keys)

Notes:

- `Runnable` is currently `Box<dyn FnOnce() + Send + 'static>` (location attribution is a future ergonomics layer).
- `Priority` is currently `DispatchPriority` (`Low/Normal/High`) and should remain extendable without breaking the base trait.

### Inbox (portable helper)

Target placement: ecosystem (e.g. `ecosystem/fret-executor` or `ecosystem/fret-bootstrap`).

Sketch:

- `struct Inbox<M>`
  - `fn sender(&self) -> InboxSender<M>`
  - `fn drain(&self) -> impl Iterator<Item = M>`

Guidelines:

- Messages are **data-only** (no `App` references).
- Draining happens at a runner-owned driver boundary.

### Executors (ergonomics layer)

Target placement: ecosystem (same crate as `Inbox`).

Sketch:

- `ForegroundExecutor` (UI thread)
  - `fn spawn_local(&self, fut: impl Future<Output = ()> + 'static) -> ForegroundTask` (`!Send`)
- `BackgroundExecutor`
  - `fn spawn(&self, fut: impl Future<Output = T> + Send + 'static) -> BackgroundTask<T>`
  - cancellation is default on drop

Important: Foreground task handles should be `!Send` to prevent cross-thread drop hazards.

## Where code changes likely land (crate-by-crate)

This section is intentionally explicit so we can track the refactor impact.

### `crates/fret-runtime`

Likely changes:

- introduce the portable `Dispatcher` trait (and any minimal supporting types).
- optionally define a small "execution capability" vocabulary that can be surfaced through
  `PlatformCapabilities` (ADR 0054).

Non-goals:

- do not depend on `winit`, `wgpu`, or any platform-specific crates.
- do not force Tokio.

### `crates/fret-launch` (desktop runner)

Likely changes:

- implement `Dispatcher` backed by the event loop proxy (`wake`) and a background scheduling mechanism.
- consolidate existing "platform completion background thread + proxy wake" logic to use the shared surface.

Care points:

- ensure shutdown paths never drop `!Send` foreground tasks on background threads.
- ensure `wake` coalescing is consistent.

### `crates/fret-runner-web` / web runner wiring

Likely changes:

- implement `Dispatcher` with:
  - `dispatch_on_main_thread` mapping to the wasm main thread queue,
  - `dispatch_after` mapping to RAF/timeout,
  - `dispatch_background` implemented as cooperative/best-effort.

### `ecosystem/fret-bootstrap` (golden path)

Likely changes:

- provide `Inbox` + executors as the default user story.
- wire inbox draining at a driver boundary inside the golden path driver.

### Ecosystem crates (third-party author story)

Migration targets (incremental, not all at once):

- replace bespoke channels/timers with:
  - inbox sender + `wake` on completion,
  - background tasks via executor adapters (optional).

## Driver boundary integration points

We need a single, easy-to-locate place where the runner/driver:

- drains inboxes,
- applies model/global updates,
- drains effects,
- schedules redraw / begins a frame.

Plan:

- define "driver boundary" in the runner loop as a first-class phase.
- expose a single hook in the golden path driver to register inbox drainers.

## Mobile readiness checklist (design-time)

Before implementing a mobile runner, ensure:

- `dispatch_on_main_thread` maps to OS main thread (UIKit/Looper).
- `wake` maps to a "reach next driver boundary" request (DisplayLink/Choreographer).
- cancellation semantics are strict (no callbacks after teardown).
- surface lifecycle boundaries are explicit (suspend/resume/surface lost).

## Documentation refactor inventory

Once the surface exists, update docs to avoid duplicated or contradictory guidance:

- `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`: replace ad-hoc "mpsc + timer" guidance with the shared surface as the default, keep the pattern as an explanation.
- `docs/adr/0113-ecosystem-integration-contracts.md`: point ecosystem authors to the dispatcher/inbox surface and document the portability traps to avoid.
- `docs/crate-usage-guide.md`: add a "Background work" section describing the recommended crates and patterns.
- `docs/effects-authoring.md`: clarify where timers live (`Effect::SetTimer`) and how they relate to `dispatch_after`.
- `docs/runtime-contract-matrix.md`: add a row for execution/wake/timers semantics by platform (native/wasm/mobile).
- `docs/ui-ergonomics-and-interop.md`: document the "heavy app" adapter story (Tokio thread + wake + inbox).

## Phased rollout plan

See `## Tracking (living TODOs)` at the top of this file for the authoritative per-phase checklist.

## Open questions (to resolve before locking v1)

- What is the minimal timer vocabulary that avoids split-brain between effects and dispatcher scheduling?
- What is the minimal "priority/backpressure" surface we want to reserve in v1?
