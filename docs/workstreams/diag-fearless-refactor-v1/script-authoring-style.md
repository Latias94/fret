---
title: Diagnostics script authoring style (stability first)
status: draft
date: 2026-02-23
scope: diagnostics, automation, scripts
---

# Script authoring style (stability first)

This doc is a set of guidelines for writing stable, agent-friendly diagnostics scripts.

Goals:

- Prefer deterministic outcomes over “pretty” evidence.
- Keep waits bounded and explainable.
- Make failures actionable (a human or agent should know what to try next).

## 1) Prefer selectors and inventories over coordinates

- Use `test_id` / `role` / stable selector primitives first.
- Avoid brittle pixel offsets unless you are explicitly testing hit-testing or layout geometry.
- If you must use coordinates, anchor them to a viewport-relative reference (and keep the click target large).

## 2) Keep waits bounded (never “wait forever”)

- Prefer “wait until predicate is true, with timeout” over fixed sleeps.
- Use the smallest timeout that is realistic across machines.
- If a wait times out, record:
  - the predicate name,
  - the last observed value(s),
  - a suggested next probe (e.g. “dump selector inventory”).

## 3) Use a two-phase pattern: warmup → measurement

- Warm up caches/state explicitly with a small number of frames/actions.
- Only start “assertions that matter” after warmup is complete.
- When comparing two snapshots, annotate which snapshot is “before” vs “after” (avoid ambiguous diffs).

## 4) Prefer structured evidence diffs

- Prefer “inventory changed” / “node state changed” / “command gating changed” style assertions over screenshot diffs.
- Keep screenshots as last resort evidence (useful for UX review, less useful for agentic triage).

## 5) Keep scripts small and composable

- Favor a small library of reusable steps:
  - `open_page(...)`
  - `focus(...)`
  - `type_text(...)`
  - `click_by_test_id(...)`
  - `scroll_into_view(...)`
  - `assert_*` predicates
- Prefer multiple short scripts over one giant script with many branches.

## 6) Make scripts forward-compatible with schema evolution

- Assume `bundle.json` may be schema v2 and semantics may be stored in a table (not inline).
- Prefer sidecar-driven triage (`frames.index.json`, `bundle.index.json`) whenever possible.
- Treat `diag doctor` as step zero for scripted repro loops.

