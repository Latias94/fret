# Closeout Audit — 2026-04-11

Status: closed closeout record

Related:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/container-aware-editor-rail-helper-shape-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`

## Verdict

This lane is now closed on an explicit outer-shell ownership verdict.

When an editor-oriented app needs to downgrade a desktop rail for mobile or compact devices:

- the outer shell owns that downgrade decision,
- the mounted inner rail remains container-aware once it exists,
- and the repo still does not justify a shared editor-rail-specific mobile downgrade helper.

## What this lane closes on

### 1) Device-shell branching and container-aware rails remain separate layers

The repo now has a closed, non-overlapping split:

- device-shell branching belongs to the existing strategy layer,
- desktop rail mounting belongs to `WorkspaceFrame.left/right`,
- inner reusable editor content belongs to `fret-ui-editor`,
- and editor-rail mobile downgrade belongs above all of that in the outer shell.

This matches ADR 0325's axis split instead of trying to force one widget family to own both
device-shell and panel/container adaptation.

### 2) `Sidebar` remains the wrong abstraction for editor-rail downgrade

The current docs already freeze that:

- `SidebarProvider::is_mobile(...)` is app-shell/device-shell vocabulary,
- forced-mobile sidebar examples only prove an app-shell sheet path,
- and sidebar mobile inference should not silently become the editor-rail answer.

So the editor-rail story closes without widening `Sidebar`.

### 3) Explicit branch composition remains the correct current posture

The repo already has one good pattern for reviewable mobile downgrade:

- desktop branch explicit,
- mobile branch explicit,
- and the policy visible at the call site.

For editor rails, that means current apps should keep the downgrade explicit in the outer shell:

- remount as `Drawer` or `Sheet`,
- route to another page/stack,
- or temporarily omit the rail,

depending on app policy.

### 4) No shared helper is justified yet

This lane does **not** extract a shared editor-rail mobile downgrade helper.

Why:

- no current editor proof surface shows repeated downgrade behavior,
- no current proof surface shows a stable shared policy beyond generic device-shell branching,
- and extracting now would mostly freeze app-local presentation policy.

## Reopen criteria

Reopen only if future evidence shows all of the following:

1. at least two editor-oriented shells share the same mobile downgrade shape,
2. the shared part is more specific than generic `device_shell_switch(...)`,
3. outer-shell ownership remains explicit,
4. and the proposed helper does not collapse app-shell/device-shell semantics back into `Sidebar`.
