# ImUi Editor Notes Dirty Status v1 - Design

Status: closed narrow P1 lane
Last updated: 2026-04-24

## Problem

`editor_notes_demo.rs` now has shell-mounted collection, inspector, and one app-owned inspector
command/status loop. The next non-multi-window editor-grade gap is local dirty/clean feedback for
notes editing: users should see whether the current notes field has a preserved draft, a committed
state, or a canceled edit without reopening shell dirty-close policy.

## Scope

Owned here:

1. Add app-owned dirty/clean status copy to the existing notes inspector surface.
2. Keep the status derived from the existing notes outcome and committed notes models.
3. Add stable test IDs/source-policy markers.

Not owned here:

1. No workspace dirty-close prompt.
2. No document persistence or save command.
3. No generic inspector, command, clipboard, or IMUI helper API.
4. No multi-window runner work.

## Target Slice

Add a visible `Draft status` row to the inspector that communicates the current app-owned notes
status using the already tracked notes outcome and committed note count.
