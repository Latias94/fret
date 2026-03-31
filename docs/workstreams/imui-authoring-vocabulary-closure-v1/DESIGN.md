# imui authoring vocabulary closure v1 - design

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: `repo-ref/imgui`
- egui: `repo-ref/egui`
- shadcn/ui: `repo-ref/ui`
- Base UI: `repo-ref/base-ui`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Active workstream

Last updated: 2026-03-31

## Purpose

The in-tree `imui` stack is no longer blocked by the old structural questions.

The repo already closed:

- the stack reset and alias cleanup,
- the editor composite / tooltip / tree / typed drag-drop helper closure,
- the reusable sortable recipe lane,
- the same-window and cross-window ghost baselines,
- and the shell-aware docking ghost / transparent overlap follow-ons.

What remains relative to Dear ImGui and egui is narrower:

- a handful of high-frequency immediate authoring nouns are still missing or underpowered,
- so dense editor lists, pickers, and data panes still require more bespoke glue than they should.

This workstream exists to close that remaining vocabulary gap without reopening the stack-reset
question and without turning `imui` into a second component policy runtime.

## Current assessment

The current direction is correct.

Fret already has the right ownership split:

- `fret-imui` stays a minimal frontend,
- `fret-ui-kit::imui` owns optional generic immediate helpers,
- `fret-ui-editor::imui` owns thin adapters over editor controls/composites,
- shell/docking choreography stays outside generic `imui`.

The current issue is not primitive scarcity.
The current issue is writer-facing vocabulary density.

Today the stack already covers:

- layout/container helpers,
- popups, menus, tooltips, floating windows, and response-query helpers,
- basic form controls,
- tree/disclosure helpers,
- typed drag/drop seams,
- and editor composites/adapters.

But compared with the everyday Dear ImGui / egui authoring loop, it still lacks a few high-traffic
surfaces:

- a generic `selectable` row/item family,
- a generic immediate `begin_combo` / `combo` family,
- a generic table/columns wrapper,
- a generic list clipper / virtualized row helper,
- and small hand-feel helpers such as `separator_text`.

## Why this should be a new lane

These remaining gaps are easy to underestimate because each one is small.

They still matter because they compound in every editor-grade screen:

- outliners,
- pick lists,
- result panes,
- palette-like popups,
- dense property tables,
- and large scrollable inspector sections.

Without these helpers, call sites keep re-solving the same problems:

- full-row hit boxes,
- stable selection row behavior,
- combo preview plumbing,
- visible-row clipping,
- and section-label chrome.

The right response is not "add many more widgets".
The right response is "close the missing high-frequency vocabulary and delete overlap".

## Relationship to adjacent lanes

This workstream is intentionally narrower than several nearby lanes:

- `docs/workstreams/imui-stack-fearless-refactor-v1/`
  - owns the stack reset and canonical ownership story.
- `docs/workstreams/imui-editor-grade-surface-closure-v1/`
  - closed editor composites, tooltip/tree helpers, and typed drag/drop seams.
- `docs/workstreams/select-combobox-deep-redesign-v1/`
  - owns shadcn/Base UI `Select` and `Combobox` part-surface parity.
- `docs/workstreams/imui-sortable-recipe-v1/`
  - owns reorder policy above the generic drag/drop seam.

This lane owns only the remaining generic immediate authoring vocabulary in `imui`.

## Goals

### G1 - Close the remaining high-frequency immediate nouns

Ship a small set of generic helpers that noticeably reduce repeated authoring glue:

- `selectable`
- `begin_combo` / `combo`
- immediate table/columns
- list clipper / virtualized rows
- `separator_text`

### G2 - Keep the owner split explicit

- generic immediate helpers belong in `ecosystem/fret-ui-kit::imui`,
- editor-specific wrappers belong in `ecosystem/fret-ui-editor::imui`,
- shell/workspace/docking policy stays outside generic `imui`.

### G3 - Prefer fearless replacement over alias growth

If a better canonical helper lands, do not keep two overlapping "default" paths alive.

Examples:

- if a generic combo helper supersedes an older narrow select convenience, either make the helper a
  tiny combo-aligned convenience or delete it,
- do not keep two first-class selection row helpers that differ only by naming.

### G4 - Lock outcomes with small gates

Each new helper must land with at least one focused regression artifact:

- unit test,
- smoke test,
- or proof/demo evidence anchor.

### G5 - Reuse existing substrate first

Do not rebuild selection, popup, scroll, or editor control state machines if the repo already has
the correct substrate.

## Non-goals

- Recreating Dear ImGui's full API grammar or flag matrix.
- Adding broad style-stack mirrors just for familiarity.
- Pulling docking tab bars, workspace shells, or tear-out choreography into generic `imui`.
- Reopening the tooltip/tree/editor composite questions already closed by earlier lanes.
- Expanding `crates/fret-ui` contracts for policy-only authoring convenience.
- Keeping compatibility aliases after the new canonical helper is clear.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `ecosystem/fret-imui` | minimal frontend, identity scopes, output collection | generic policy-heavy helpers, editor composites, shell choreography |
| `ecosystem/fret-ui-kit::imui` | generic immediate authoring vocabulary (`selectable`, combo, table, clipper, section helpers) | editor-only composites, docking/workspace policy, broad compatibility alias matrices |
| `ecosystem/fret-ui-editor::imui` | thin adapters that are clearly editor-specific | duplicate generic immediate helpers, adapter-local state machines that should live below |
| `ecosystem/fret-ui-shadcn` | shadcn/Base UI/Radix recipe surfaces | generic immediate helper ownership |
| `crates/fret-ui` | mechanisms/contracts | immediate ergonomics and policy-only authoring sugar |

## Prioritized closure set

### P0 - `selectable` row/item family

Target outcome:

- a generic immediate row helper suitable for list selection, menu-like rows, outliners, and popup
  option lists,
- with stable ids, full-row hit testing, selected/disabled state, and explicit popup-close policy.

Why first:

- Dear ImGui uses `Selectable()` everywhere,
- egui uses `ui.selectable_value(...)` / `SelectableLabel` everywhere,
- and many follow-on helpers become cleaner once row selection has a canonical immediate surface.

### P0 - generic `begin_combo` / `combo` family

Target outcome:

- a generic immediate choice-picker helper with preview text and popup body ownership,
- usable for dense editor pickers without forcing shadcn/Base UI part-surface machinery,
- and flexible enough to host `selectable(...)` rows or small custom row bodies.

Clarification:

- this is not the same as the shadcn/Base UI `Combobox` lane,
- and it should not duplicate recipe-owned part surfaces.

### P1 - generic immediate table/columns wrapper

Target outcome:

- a table-like immediate helper for headers + rows + columns,
- not just a plain layout grid,
- and not a full data-grid or spreadsheet contract.

### P1 - generic list clipper / virtualized row helper

Target outcome:

- a small immediate helper for large uniform or near-uniform lists,
- with stable keyed rows and visible-range submission,
- reusable outside editor-only `PropertyGridVirtualized`.

### P2 - `separator_text` and small hand-feel helpers

Target outcome:

- close the remaining small-but-frequent chrome helpers after the higher-value surfaces are stable.

## Design stance for the current helpers

### model-backed single-select convenience is allowed only as a thin combo wrapper

A model-backed single-select helper can still be worth keeping.

But it must be obviously downstream of the canonical combo surface:

- naming should align with `combo`,
- implementation should be a thin wrapper over `combo(...)`,
- and it must not preserve a second parallel select/popup abstraction.

### `grid` is not the same thing as `table`

`grid(...)` already solves layout.
It does not by itself solve:

- header/body coordination,
- row-oriented authoring,
- column semantics,
- or large-table clipping expectations.

### editor virtualization does not close the generic clipper question

`PropertyGridVirtualized` is useful proof that virtualization can work in the editor layer.
It is not a generic immediate list clipper for `imui`.

## Gate expectations

Minimum deliverables per helper:

- one focused implementation anchor,
- one focused regression artifact,
- and one proof surface that uses the helper in a real immediate authoring context.

Preferred proof surfaces:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- focused `fret-ui-kit` smoke tests
- `fret-imui` interaction tests where response semantics matter

## Definition of done

This workstream is complete when:

- the remaining missing high-frequency `imui` vocabulary is closed or explicitly deferred,
- no historical helper lane needs to be reopened for generic immediate authoring,
- the owner split remains clean,
- and the new surfaces are teachable without compatibility aliases.
