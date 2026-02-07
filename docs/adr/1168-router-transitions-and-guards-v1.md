# ADR 1168: Router Transitions, Events, and Guards (v1)

Status: Proposed

## Context

Fret targets “editor-grade UI” applications where navigation state, deep links, and history are
first-class concerns. In `ecosystem/fret-router`, we want a TanStack-inspired router core that is
portable (native + wasm) and stable enough to be adopted by multiple apps without repeated API
rewrites.

The highest risk surfaces are:

- transition semantics (`push`/`replace`/`back`/`forward` vs redirects vs sync),
- guard outcomes (allow/block/redirect),
- deterministic diagnostics (so repro bundles can encode what happened),
- history adapter constraints (web history cannot “peek” the next back/forward entry).

This ADR locks the minimal v1 contract for transitions, events, and guards.

## Decision

### 1) Transition snapshots are explicit and diagnostics-friendly

`ecosystem/fret-router` provides a portable transition snapshot:

- `RouterTransitionCause`:
  - `Navigate { action }`: the router performed a normal navigation
  - `Redirect { action }`: the router navigated to a different target due to guard policy
  - `Sync`: the router state changed due to external history changes (e.g. web `popstate`)
- `RouterTransition` includes:
  - `from` and `to` canonical `RouteLocation`
  - `redirect_chain`: a list of attempted locations (v1: at least 0..1; multi-hop is future work)
  - `blocked_by`: optional `RouterBlockReason` when a guard blocks an attempt

`RouterUpdate` is returned from `Router::navigate(...)` / `Router::sync()`:

- `NoChange`: no observable state change
- `Changed(transition)`: router state updated and a `Transitioned` event is emitted
- `Blocked(transition)`: guard blocked an attempted transition; state remains on a safe location

### 2) Router events are a deterministic queue

The router records an ordered event queue and exposes:

- `Router::take_events() -> Vec<RouterEvent<R>>`

Events are:

- `Transitioned { transition, state }`
- `Blocked { transition, state }`

This is intentionally pull-based in v1. Subscription-based APIs can be added later once the event
surface proves stable.

### 3) Guards are an optional policy hook (app/ecosystem-owned)

The router supports an optional guard:

- `Router::set_guard(Option<RouterGuardFn<R>>)`
- `RouterGuardDecision`: `Allow` / `Block { reason }` / `Redirect { action, to }`

Guards are a policy surface and are expected to live in app/ecosystem layers (e.g. auth,
permissions, onboarding). The router core provides only the mechanism and the stable outcome
representation.

### 4) Guard evaluation rules by navigation type

#### Push / Replace (pre-guard)

For `Push` and `Replace`, the router always evaluates the guard **before** mutating history.

- `Allow`: perform navigation
- `Block`: do not mutate history; emit `Blocked`
- `Redirect`: navigate to `to` with `action` (normalized; see below); emit `Transitioned` with
  `cause = Redirect { action }` and `redirect_chain = [attempted]`

#### Back / Forward (peek-preferred, post-guard fallback)

Back/Forward navigation depends on adapter capabilities:

- If `HistoryAdapter::peek(action)` returns a `RouteLocation`, the router runs a **pre-guard** and
  can block or redirect without mutating history.
- If `peek` is unavailable, the router performs navigation first, then evaluates the guard as a
  **post-guard**:
  - `Allow`: keep the new location
  - `Redirect`: navigate to the redirected location and emit a redirect transition
  - `Block`: perform a *soft block* by executing `Replace(from)` to restore a safe location, then
    emit `Blocked`

This design keeps web adapters usable while still allowing deterministic pre-guard behavior for
portable/native adapters.

### 5) Redirect action normalization

Redirect decisions normalize invalid redirect actions:

- `Back` / `Forward` redirects are treated as `Replace` to avoid ambiguous “redirect by history
  traversal” semantics.

### 6) Open questions (tracked, not blocked)

v1 intentionally leaves these as follow-ups:

- Multi-hop redirect chains, loop detection, and hop limits.
- Async guard outcomes (awaitable policies) and cancellation semantics.
- `serde`-friendly transition/event snapshots for diagnostics bundles.

## Alternatives considered

### A) Require `peek` for all adapters

Rejected: web history cannot reliably provide “peek next back/forward location”, and forcing it
would make wasm adoption worse.

### B) Do not guard Back/Forward

Rejected: editor-grade apps need predictable auth/permission gating even when users navigate with
Back/Forward.

### C) Always post-guard Back/Forward (no peek)

Not chosen as the only option: post-guard works for web, but peek-pre-guard provides better
determinism and avoids unnecessary history mutations when available.

## Evidence anchors (current implementation)

- Router transitions/events/guards: `ecosystem/fret-router/src/router_state.rs`
- Workstream tracking: `docs/workstreams/router-tanstack-parity-v1.md`
- Workstream TODOs: `docs/workstreams/router-tanstack-parity-v1-todo.md`

