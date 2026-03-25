---
title: Clipboard Write Completion (Fearless Refactor v1) — Milestones
status: draft
date: 2026-03-25
scope: clipboard, runtime contract, diagnostics, ai elements parity, refactor
---

# Clipboard Write Completion (Fearless Refactor v1) — Milestones

Workstream entry:

- `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/richer-clipboard-surfaces.md`

## M0 — Contract direction is locked

Definition of done:

- the final public naming is chosen,
- the workstream explicitly commits to either:
  - the full fearless reset (`Read/Write` rename + write completion), or
  - a narrower non-renaming variant,
- ADR follow-up scope is clear before code churn starts.

## M1 — Runtime contract exists

Definition of done:

- `fret-core` exposes clipboard write completion outcomes,
- `fret-runtime` exposes the new clipboard write effect,
- write failures are no longer diagnostics-only knowledge,
- the new contract is window-aware and token-addressable,
- the text-first lane and future payload lane are both explicit in the contract story.

## M2 — Runner + diagnostics alignment

Definition of done:

- desktop and web runners emit the same clipboard write completion semantics,
- diagnostics snapshots record clipboard write results from the real completion flow,
- clipboard failure simulation docs describe both read and write denial paths.

## M3 — `fret-ui` routing primitive lands

Definition of done:

- components can register clipboard completion handlers without a generic event bus,
- token routing is covered by focused tests,
- the mechanism surface stays narrow and reusable.

## M4 — First-party callers are honest

Definition of done:

- AI copy buttons only report success after real write completion,
- failure callbacks are exposed where upstream semantics require them,
- copied/check UI state no longer appears on forced write failure,
- non-AI clipboard write callers are migrated.

## M5 — Legacy surface is deleted

Definition of done:

- `ClipboardSetText` / `ClipboardGetText` are removed from the shipped public surface,
- UI Gallery/docs no longer describe clipboard copy as fire-and-forget for explicit copy buttons,
- ADR alignment and regression gates are updated,
- the new surface is the only one first-party docs teach,
- future rich clipboard support can land additively without renaming the contract again.
