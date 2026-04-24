# ImUi Editor Notes Inspector Command v1 - Design

Status: active narrow P1 lane
Last updated: 2026-04-24

## Problem

The helper-readiness lane closed without shared collection helper growth. The next useful
non-multi-window improvement should therefore deepen app-owned editor-grade behavior in an existing
proof surface instead of widening `fret-ui-kit::imui`.

`editor_notes_demo.rs` already proves shell-mounted collection and inspector rails. It can now prove
a small inspector-local command/status loop that feels editor-grade without introducing a generic
command palette, clipboard service, or inspector helper.

## Scope

Owned here:

1. Add one explicit inspector-local command affordance to `editor_notes_demo.rs`.
2. Keep command state and status copy app-owned.
3. Add stable test IDs for the command action and status evidence.
4. Preserve the existing `InspectorPanel` / `PropertyGrid` composition.

Not owned here:

1. No `fret-ui-kit::imui` helper widening.
2. No generic command palette or command bus.
3. No platform clipboard integration.
4. No `crates/fret-ui` runtime/mechanism change.
5. No multi-window runner work.

## Target Slice

Add a `Copy asset summary` inspector command that updates an app-owned status model with a summary
of the selected asset. The command is intentionally local: it proves a reviewable editor command
loop without claiming a shared command abstraction.
