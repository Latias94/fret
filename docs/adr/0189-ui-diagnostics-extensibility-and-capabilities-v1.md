---
title: "ADR 0189: UI Diagnostics Extensibility + Capabilities v1"
---

# ADR 0189: UI Diagnostics Extensibility + Capabilities v1

Status: Proposed

Scope: the long-lived *contract shape* for scripted UI automation and diagnostics workflows, with an explicit focus on:

- ecosystem author ergonomics (apps/components outside the core workspace),
- future target surfaces (multi-window, embedded viewports/canvas, and potential mobile runners),
- protocol evolution rules (versioning + capability negotiation).

Related:

- ADR 0159 (bundles + scripts baseline): `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- ADR 0181 (automation + debug recipes): `docs/adr/0181-ui-automation-and-debug-recipes-v1.md`
- ADR 0033 (semantics / a11y selectors): `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- ADR 0017 (multi-window / DPI): `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0238 (pointer coordinate spaces): `docs/adr/0238-pointer-coordinate-spaces-and-element-local-mapping-v1.md`

Non-normative:

- Living usage doc: `docs/ui-diagnostics-and-scripted-tests.md`

## Context

Fret already has a usable diagnostics + scripted automation workflow (ADR 0159 / ADR 0181) that is effective for
in-repo development. The next scaling step is to make the automation surface:

1) easier to author and refactor (especially for large scenario suites),
2) robust across UI refactors (selectors and intent-level steps),
3) explicit about optional capabilities (screenshots, multi-window, gestures),
4) evolvable without breaking older scripts.

Other ecosystems (browser automation, native UI testing, Flutter widget testing) converge on a common lesson:

- if automation relies on coordinates, it becomes brittle;
- if optional features are not negotiated, scripts fail as timeouts and are hard to debug;
- if versioning rules are unclear, “small additions” quickly become breaking changes.

## Goals

1. Define a forward-compatible shape for scripted interaction tests (selectors, actions, assertions) that can cover:
   - single window,
   - multi-window,
   - embedded viewports/canvas-style hit testing,
   - touch/gesture surfaces (future).
2. Make “what this runner supports” explicit via capabilities and enforce a predictable failure mode.
3. Keep the core runtime policy-free (ADR 0066): runtime provides minimal hooks and versioned data; tooling owns policy.
4. Preserve JSON scripts as the portable, reviewable artifact, even when authored via typed helpers.

## Non-goals

- Screenshot goldens as the primary correctness mechanism.
- A general-purpose “execute arbitrary app code” step in the protocol.
- Guaranteeing perfect IME determinism across platforms.
- Mandating a single CLI/tool UX; this ADR defines protocol-level contracts, not a specific UI.

## Decision

### 1) Treat the script JSON as the primary portable artifact

Scripts MUST remain serializable as JSON and SHOULD be small, reviewable units:

- one scenario per script,
- stable selectors (`test_id` or semantics role/name),
- deterministic waits (`wait_until` predicates, not arbitrary sleeps).

Typed authoring helpers (e.g. Rust builders) are allowed, but they MUST compile down to the same JSON contract.

Rationale:

- JSON scripts can be reviewed and diffed easily,
- scripts can be executed by multiple transports (file-trigger and devtools WS),
- scripts can be packed and shared with diagnostics bundles for AI/human triage.

### 2) Introduce explicit capability negotiation

The tooling stack MUST support a “capabilities” concept to avoid “hang until timeout” failure modes.

Minimum capability vocabulary (v1):

- `diag.script_v2`: supports Script schema v2 steps.
- `diag.screenshot_png`: supports on-demand PNG screenshots requested by scripted steps.
- `diag.multi_window`: supports explicit window targeting and cross-window assertions.
- `diag.pointer_kind_touch`: supports touch pointer injection (future runner-dependent).
- `diag.gesture_pinch`: supports pinch/zoom gestures (future runner-dependent).

Tooling SHOULD accept the un-namespaced aliases (`script_v2`, `screenshot_png`, etc) as backwards-compatible inputs,
but runners SHOULD advertise the namespaced `diag.*` forms.

Behavior:

- A script MAY declare `required_capabilities`.
- If any required capability is missing, execution MUST fail fast with a structured reason (tooling-side), not time out.

Transport notes:

- For devtools WS transports, capabilities SHOULD be negotiated via a hello/ack handshake.
- For file-trigger transports, capabilities MAY be declared by the runner in a sidecar file under `FRET_DIAG_DIR`
  (e.g. `capabilities.json`) so tooling can read them deterministically.

### 3) Add optional script metadata as a stable extension point

Scripts MAY include an optional `meta` object (schema-stable):

- `name`: human-readable stable name,
- `tags`: small set of strings (e.g. `smoke`, `overlay`, `ime`, `canvas`),
- `required_capabilities`: list of capability strings,
- `target_hints`: optional non-normative hints (e.g. “run against ui_gallery”, “requires view cache on”).

Rules:

- Tooling MUST ignore unknown keys under `meta` to allow future expansion.
- `meta` MUST NOT affect execution semantics directly; it is for orchestration and discovery only.

### 4) Extend scripts to target windows explicitly (future-proofing)

To support multi-window, steps and predicates SHOULD gain an optional `window` field:

- absent: operate on the current active window (today’s behavior),
- present: resolve targets and perform actions within the selected window.

Window selector (v1 shape, future implementation):

- by stable `window_id` when available,
- or by `title`/`role_and_name` semantics when feasible.

This allows tests like:

- open dialog in window A,
- move focus to window B,
- assert a barrier root relationship remains valid per window.

### 5) Canvas and embedded viewport automation: prefer semantics projections

For embedded viewports/canvas:

- preferred: project canvas-interactive objects into the semantics tree with `test_id` (ADR 0033), then automate via
  standard selectors and intent-level steps;
- fallback: allow coordinate-based actions relative to a selected element bounds (not absolute screen coordinates).

Coordinate fallback rules:

- coordinates MUST be expressed in a clearly defined space:
  - `window_local` or `element_local_normalized` (0..1 in both axes),
  - never raw “screen pixels” without a selector anchor.
- steps using coordinate injection MUST be capability-gated (e.g. `pointer_injection`).

### 6) Protocol evolution rules

- Additive changes MUST be backward compatible within a major schema version:
  - new optional fields,
  - new step variants (when the schema version changes, or when tooling can negotiate via capabilities).
- Breaking changes MUST increment `schema_version`.
- Tooling SHOULD provide “print normalized JSON” for scripts to keep diffs stable and minimize merge conflicts.

## Implementation notes (non-normative)

As of 2026-02-10, the repo provides:

- JSON scripts in `tools/diag-scripts/` executed via `fretboard diag`.
- Script schema v2 intent-level steps in `crates/fret-diag-protocol`.
- Typed script authoring helpers (builder) and an internal script generator tool for ergonomic authoring.

These are compatible with the ADR as long as:

- JSON remains the portable artifact,
- the protocol remains versioned,
- optional capabilities are surfaced and enforced deterministically.
