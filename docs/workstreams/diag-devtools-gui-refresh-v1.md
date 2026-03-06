---
title: Diagnostics DevTools GUI Refresh v1
status: draft
date: 2026-03-06
scope: diagnostics, devtools, gui, ux, product
---

# Diagnostics DevTools GUI Refresh v1

This workstream defines a focused product/UX refresh for `apps/fret-devtools`.

It is a follow-up to the existing DevTools GUI workstream:

- baseline architecture and transport: `docs/workstreams/diag-devtools-gui-v1.md`
- current dogfood path: `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`

The purpose of this note is not to change diagnostics contracts.

The purpose is to make the current DevTools GUI feel like a tool maintainers can keep open every day, while
still remaining a thin consumer of the same diagnostics model used by CLI and MCP.

## Problem statement

The current DevTools GUI already has meaningful capability:

- inspect and pick,
- script loading/editing/running,
- bundle packing,
- regression aggregate browsing,
- summary drill-down,
- summarize and selected-evidence pack actions.

However, the current presentation is still closer to an internal diagnostics console than a productized
developer tool.

Current pain points:

- too much information appears with nearly equal visual weight,
- large text blobs dominate the viewport too early,
- the primary user journey is not visually obvious,
- action groups are technically present but weakly staged,
- aggregate status and evidence actions are discoverable only after reading closely,
- the app reads more like a feature dump than a guided workflow.

## Design goals

### 1. Make the primary path obvious

The most important everyday path should read clearly as:

1. Inspect / Pick
2. Choose or patch script
3. Run
4. Summarize
5. Inspect failing summary
6. Pack / share evidence

The GUI should visually teach this path without requiring documentation first.

### 2. Reduce first-open cognitive load

On first open, the app should answer:

- what is connected,
- what can I do next,
- where the current session/artifacts root is,
- whether the latest run passed or failed,
- how to reach one concrete failing evidence item quickly.

### 3. Keep raw diagnostics detail available but secondary

Raw text and JSON remain valuable, but they should be behind a clear layer boundary:

- summary-first,
- detail second,
- raw payload last.

### 4. Stay contract-faithful

This refresh must not introduce:

- a GUI-only regression schema,
- a GUI-only campaign state machine,
- hidden evidence paths,
- a second interpretation of diagnostics status.

## Proposed information architecture

## Top-level layout

Prefer a three-band structure:

1. **Command bar**
   - transport/session status,
   - inspect/pick actions,
   - run/summarize actions,
   - compact state badges.
2. **Primary workspace**
   - left: navigation / lists / script library / failing summaries,
   - center: current task surface,
   - right: details / evidence / raw payload tabs.
3. **Support rail**
   - logs,
   - low-priority raw text blobs,
   - debugging metadata.

The main idea is simple: actionable surfaces should live above inspection payloads.

## Primary tabs

The current app can keep multiple details tabs, but the default experience should be organized around four
clear task areas:

### 1. Inspect

Purpose:

- turn a live UI target into a stable selector,
- review semantics-driven node details,
- move quickly into script authoring.

Key surfaces:

- inspect enable/disable,
- arm pick,
- latest pick result,
- apply-pick affordance,
- selected node summary.

### 2. Scripts

Purpose:

- choose an existing script,
- patch a step,
- run it,
- see immediate run status.

Key surfaces:

- script list,
- editor,
- validation summary,
- run / run-and-pack actions,
- last run status line.

### 3. Regression

Purpose:

- treat aggregate artifacts as a first-class diagnostics workspace,
- go from summary to one failing evidence payload quickly.

Target structure:

- top summary strip:
  - loaded artifacts root,
  - summarize status,
  - summary counters/badges,
  - refresh/summarize actions.
- middle split:
  - left: failing summaries list,
  - right: selected summary detail.
- bottom or side drawer:
  - raw aggregate payloads,
  - raw selected summary JSON,
  - copyable evidence paths.

This tab should evolve toward a master-detail flow, not a stack of text cards.

### 4. Evidence

Purpose:

- make evidence handoff explicit and low-friction.

Key surfaces:

- latest bundle/pack path,
- copy actions,
- selected evidence actions,
- open viewer entry,
- future pack history.

## Visual language refresh

## Hierarchy

- stronger titles for task areas,
- compact descriptions under titles,
- badges for live state instead of long status sentences where possible,
- primary actions visually distinct from utility actions.

## Density

- reduce the amount of full-width raw text visible at once,
- prefer lists, stats, badges, and compact rows before raw dumps,
- reserve scrollable text areas for drill-down and troubleshooting.

## Grouping

- keep actions near the data they affect,
- avoid mixing inspect actions, run actions, and evidence actions in one undifferentiated row,
- visually separate "do work" actions from "copy/export" actions.

## Phase plan

## Phase A — Information architecture cleanup

Deliverables:

- explicit command bar grouping,
- clearer tab naming,
- top-level status strip,
- logs moved to a lower-priority region.

Acceptance:

- a first-time contributor can identify the next action in under 10 seconds,
- the most common commands are visible without reading raw logs.

## Phase B — Regression workspace refresh

Deliverables:

- master-detail failing summaries layout,
- compact aggregate stats strip,
- evidence actions grouped near selected summary,
- raw JSON moved behind secondary affordances.

Acceptance:

- a maintainer can move from aggregate failure to one packed evidence artifact with minimal scanning,
- the regression tab no longer reads like three independent debug blobs.

## Phase C — Script Studio polish

Deliverables:

- stronger loaded-script context,
- clearer validation/run state,
- more obvious relationship between pick/apply/run.

Acceptance:

- script authoring feels like one coherent workflow rather than several adjacent utilities.

## Phase D — Evidence and handoff polish

Deliverables:

- clearer pack status/history,
- stronger viewer handoff surface,
- optional future "recent evidence" list.

Acceptance:

- evidence export is obvious and repeatable for human triage.

## Implementation constraints

- keep `apps/fret-devtools` as a thin consumer over diagnostics contracts,
- prefer additive layout refactors over wholesale rewrites,
- do not move policy-heavy behavior into contract crates,
- land GUI refresh in small reviewable slices,
- preserve existing dogfood workflow while improving presentation.

## Recommended landing order

1. restructure the `Regression` tab into a clear master-detail surface,
2. add a compact top status strip for transport/session/artifacts root,
3. reduce raw text dominance by collapsing or demoting low-priority blobs,
4. regroup command rows by workflow stage,
5. polish Script Studio after Regression becomes readable.

## Definition of done

This refresh is successful when:

- the DevTools GUI no longer feels like a raw diagnostics console,
- the primary inspect → script → summarize → evidence path is visually obvious,
- regression browsing is summary-first and evidence-oriented,
- no new GUI-only diagnostics model is introduced,
- the product surface looks deliberate enough to dogfood daily.
