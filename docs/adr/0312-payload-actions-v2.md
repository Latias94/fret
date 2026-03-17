# ADR 0312: Payload (Parameterized) Actions v2

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed (GPUI): https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Proposed

## Context

Action-first authoring v1 (ADR 0307) standardizes on **unit actions**:

- `ActionId` is compatible with existing `CommandId` strings,
- keymap, menus, command palette, and pointer activation converge on dispatching the same stable ID,
- diagnostics can explain availability and dispatch decisions.

In practice, editor-grade UI also needs **parameterized actions**:

- per-item actions in lists/tables/tree views (e.g. close tab `{tab_id}`),
- action handlers that need an additional input without allocating one-off models,
- keeping the dispatch pipeline observable without reintroducing ad-hoc routers.

Legacy MVU/message routing can model payloaded actions today, but it is intentionally quarantined
after v1. To shrink that legacy surface (M8), we need an action-first payload story that remains:

- portable (desktop + wasm),
- deterministic enough for scripted diagnostics,
- layered (mechanism in `crates/*`, policy in `ecosystem/*`),
- compatible with future data-driven frontends (GenUI / potential DSL).

## Goals

1. Provide a **minimal** payload action mechanism that works for pointer/programmatic dispatch.
2. Preserve v1 cross-surface alignment for unit actions (no keymap schema churn).
3. Keep the mechanism object-safe for stored handlers (ADR 0074).
4. Preserve explainability in diagnostics (ADR 0159).
5. Keep the UI IR data-first: payload is **not** embedded as arbitrary closures in the element tree.

## Non-goals (v2 prototype)

- Adding payload support to keymap JSON format (shortcuts remain unit actions).
- Serializing arbitrary Rust payloads for replay; payloads are best-effort and primarily for
  in-process dispatch.
- Replacing models/selectors/queries; payload actions are not a substitute for state management.

## Decision

### D1) Define a payload action as “ActionId + transient payload”

A payload action dispatch consists of:

- an `ActionId` (stable ID),
- a **transient payload value** associated with the next dispatch of that action.

Payloads are stored in a **window-scoped pending payload service** with a small tick TTL, similar
to `WindowPendingCommandDispatchSourceService` (diagnostics metadata).

This keeps the UI IR data-first while enabling per-item dispatch without ad-hoc routers.

### D2) Payload actions are pointer/programmatic-first in v2

v2 supports payloads for:

- pointer-triggered activation (e.g. clicking a tab close button),
- programmatic dispatch (e.g. a script driver or internal logic).

Keymap, menus, and command palette remain **unit-action** surfaces in v2:

- they dispatch an `ActionId` only,
- they do not provide payload.

Rationale: payload in keymap introduces schema + UX questions (input prompts, serialization,
replay). That is out of scope for this prototype.

### D3) Object-safe host API: record + consume payload by (window, action)

The mechanism layer exposes two object-safe operations on the action host:

- record a pending payload for a given `(window, ActionId)`,
- consume the most recent pending payload for a given `(window, ActionId)` within TTL.

Typed conveniences (downcast to `T`) are provided as extension traits in ecosystem.

### D4) Determinism and failure semantics

Normative semantics:

- payload is **best-effort** and transient; it may be dropped if it expires (TTL) or if a later
  dispatch consumes it first,
- payload actions must remain safe when payload is missing:
  - handlers should treat missing/invalid payload as “not handled” (recommended default),
  - diagnostics should allow correlating “payload expected but missing” when feasible.

This matches the “fearless refactor” posture: do not crash; keep it diagnosable.

### D5) DSL/GenUI forward-compat: payload is a separate channel, not a closure

To keep space for data-driven frontends:

- the element tree binds an `ActionId` (stable string),
- payload is provided via a separate, transient channel at dispatch time.

Future frontends can choose how to produce payload:

- static (literal) payloads for per-item actions,
- references to app-provided models/globals (by stable handle),
- or explicit prompts (future work).

## Contract shape (illustrative)

```rust,ignore
// v2: typed payload action marker
pub trait TypedPayloadAction: fret_runtime::TypedAction {
    type Payload: Any + Send + Sync + 'static;
}

// v2: record payload + dispatch action
pressable
  .action(act::WorkspaceTabClose)
  .action_payload(TabId(42));

// v2: handler consumes the pending payload
use fret::advanced::AppUiRawActionExt as _;

cx.on_payload_action_notify::<act::WorkspaceTabClose>(|host, _acx, tab_id: TabId| {
    // close the tab
    close_tab(host, tab_id);
    true
});
```

Notes:

- The exact method names are not normative; the contract is:
  - stable ID binds in IR,
  - payload channel exists and is transient,
  - consumption happens in the handler path.

## Evidence expectations

1. At least one in-tree demo migrates from MVU payload routing to payload actions.
2. Diagnostics can still explain:
   - which `ActionId` was dispatched,
   - the dispatch source (pointer/shortcut/etc.),
   - whether a payload was present (best-effort).

## Alternatives considered

### A) Encode payload in the element tree as closures

Rejected: violates IR-first/data-first posture and makes future DSL/frontends harder.

### B) Encode payload by allocating per-item unique `ActionId`s

Rejected: explodes action registry and breaks keymap/palette discoverability.

### C) Require payload to be serialized bytes everywhere

Deferred: replayable/serializable payloads are useful, but forcing it in v2 increases scope and
cost. This prototype keeps payload as `Any` and can add serialization later for specific payload
types/frontends.
