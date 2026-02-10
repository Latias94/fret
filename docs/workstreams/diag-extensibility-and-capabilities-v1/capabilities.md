---
title: Diagnostics Extensibility + Capabilities v1 - Capabilities
status: draft
date: 2026-02-10
scope: diagnostics, automation, protocol, capabilities, transports
---

# Diagnostics Extensibility + Capabilities v1 - Capabilities

This document is a sub-part of `docs/workstreams/diag-extensibility-and-capabilities-v1.md`.

Goal: make “what is supported” explicit so missing support **fails fast with a structured reason**, rather than
degrading into timeouts.

## Terminology

- **DevTools API capabilities**: what the control plane supports (inspect/pick/scripts/bundles/sessions).
- **Diagnostics/runner capabilities**: what the target runner can actually do for scripted execution (script schema,
  on-demand screenshots, window targeting, pointer injection, IME hooks, etc).

These two sets evolve independently. Mixing them under the same un-namespaced strings is a long-term foot-gun.

## Capability namespaces (recommended)

Use a single `Vec<String>` transport field, but require namespaces:

- `devtools.*`: control plane surface (e.g. `devtools.sessions`, `devtools.inspect`).
- `diag.*`: script/runner surface (e.g. `diag.script_v2`, `diag.screenshot_png`).

Rule: tooling MUST treat unknown capability strings as opaque (forward compatible).

## Initial `diag.*` vocabulary (v1)

Minimum set (aligns with ADR 0204 intent, but namespaced):

- `diag.script_v2`: Script schema v2 steps supported.
- `diag.screenshot_png`: on-demand PNG screenshot requests supported (used by `capture_screenshot`).
- `diag.multi_window`: explicit window targeting supported.

Optional (future-proofing, gate first):

- `diag.pointer_injection`: coordinate-based pointer injection supported (see canvas fallback rules).
- `diag.pointer_kind_touch`: touch pointer injection supported.
- `diag.gesture_pinch`: pinch/zoom gesture steps supported.
- `diag.text_ime_trace`: IME/composition evidence available in bundles/triage (not a step).

## Where capabilities live

### 1) Script metadata (declared requirements)

Scripts SHOULD be allowed to declare requirements via a stable metadata surface:

- `meta.required_capabilities: string[]`

Tooling MUST ignore unknown `meta` keys.

### 2) Runner discovery (available capabilities)

Tooling MUST be able to learn runner capabilities deterministically.

Two supported transports:

- **Filesystem-trigger transport**: runner writes `capabilities.json` under `FRET_DIAG_DIR`.
- **DevTools WS transport**: capabilities are negotiated as part of hello/ack and/or session descriptor.

#### `capabilities.json` (filesystem)

Recommended shape:

```json
{
  "schema_version": 1,
  "runner_kind": "native|web|unknown",
  "runner_version": "x.y.z",
  "capabilities": ["diag.script_v2", "diag.screenshot_png"]
}
```

Rules:

- must be written deterministically (startup or ready-touch time),
- must be safe to parse without launching the app (CI friendliness),
- tooling must treat unknown keys as ignorable.

#### DevTools WS hello/ack

Rules:

- the app hello SHOULD advertise both `devtools.*` and `diag.*` strings,
- tooling SHOULD select a session based on session descriptors and read capabilities from the session record.

## Gating rules (fail fast)

Given:

- `required = declared_required(script) ∪ inferred_required(script)` (tooling may infer from steps),
- `available = runner_capabilities`,

Then:

- if `missing = required - available` is non-empty:
  - execution MUST fail fast (tooling-side),
  - tooling MUST emit a structured evidence file for CI/AI.

### Evidence file for gating failures

Recommended output: `check.capabilities.json`:

```json
{
  "schema_version": 1,
  "status": "failed",
  "missing": ["diag.screenshot_png"],
  "required": ["diag.screenshot_png", "diag.script_v2"],
  "available": ["diag.script_v2"],
  "source": "filesystem|devtools_ws"
}
```

This file is a stable contract for CI and for automation/AI triage.

## Mapping steps to inferred requirements (non-normative)

Tooling can infer required capabilities from the presence of step variants:

- `capture_screenshot` ⇒ `diag.screenshot_png`
- steps that explicitly target non-default windows ⇒ `diag.multi_window`
- coordinate-based steps (if added) ⇒ `diag.pointer_injection`

Note: inference is “best effort”; `meta.required_capabilities` remains the explicit escape hatch.

