# M0 Baseline Audit — 2026-04-11

Status: baseline audit

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Assumptions-first read

### 1) Device/mobile branching is already a separate adaptive axis

- Evidence:
  - `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
  - `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would incorrectly freeze an owner split on top of an unresolved taxonomy.

### 2) Editor rails already have their desktop shell seam

- Evidence:
  - `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - mobile downgrade would still be blocked on the desktop mounting surface itself.

### 3) Current editor-rail proofs do not yet show repeated mobile downgrade behavior

- Evidence:
  - `docs/workstreams/container-aware-editor-rail-helper-shape-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - a shared helper might already be visible and this lane should not close on a no-helper verdict.

### 4) The repo already teaches one correct explicit device-shell pairing pattern

- Evidence:
  - `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
  - `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would need a fresh proof surface before making an owner recommendation.

## Audited evidence

### 1) `responsive_dialog` stays explicit about desktop/mobile shells

The current gallery proof keeps:

- desktop `Dialog`,
- mobile `Drawer`,
- and both branches visible in one snippet.

This matters because it demonstrates the intended reviewability rule:

- outer-shell downgrade branches may stay explicit instead of being hidden behind a generic widget.

### 2) `Sidebar` already documents the app-shell/device-shell boundary

The current sidebar page explicitly says:

- `SidebarProvider::is_mobile(...)` and `is_mobile_breakpoint(...)` are app-shell/device-shell
  controls,
- `Sidebar` should stay an app-shell surface,
- and forced-mobile sidebar examples do not prove editor-rail adaptation.

This gives the editor-rail story a clear negative boundary:

- `Sidebar` mobile inference is not the reusable answer for editor rails.

### 3) The editor-rail lanes already froze the desktop owner split

The previous rail lanes close on:

- shell placement through `WorkspaceFrame.left/right`,
- inner content ownership through `fret-ui-editor`,
- no public rail extraction yet,
- and no shared helper yet.

What remains for mobile is therefore narrower:

- the outer shell decides whether the rail remains mounted, remounts as a drawer/sheet, or becomes
  another page/route.

## Baseline verdict

The repo evidence supports one clear owner split:

- outer shell owns editor-rail mobile downgrade composition,
- inner rail content remains container-aware once mounted,
- and no shared editor-rail mobile downgrade helper is justified yet.
