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
- `docs/workstreams/device-shell-adaptive-facade-promotion-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
- `apps/fret-ui-gallery/tests/device_shell_recipe_wrapper_surface.rs`

## Verdict

This lane is now closed.

The repo does not need a second recipe-owned device-shell wrapper today.

The shipped posture is:

- keep `Combobox::device_shell_responsive(...)` as the current recipe-owned wrapper exemplar,
- keep `Date Picker` and `Breadcrumb` on the explicit shared-helper app/gallery lane,
- keep `Dialog` vs `Drawer` as an explicit docs/proof pairing,
- keep `SidebarProvider::is_mobile(...)` on the app-shell/provider boundary,
- and align the existing `Combobox` wrapper internals with the shared helper owner instead of
  duplicating raw viewport-switch logic.

## What shipped

### 1) `Combobox` now delegates binary shell classification to the shared helper owner

`ecosystem/fret-ui-shadcn/src/combobox.rs` still exposes the same recipe-owned public API:

- `device_shell_responsive(bool)`
- `device_shell_md_breakpoint(Px)`

But its internal desktop/mobile classification now delegates to:

- `fret_ui_kit::adaptive::DeviceShellSwitchPolicy`
- `fret_ui_kit::adaptive::device_shell_mode(...)`

This keeps the wrapper recipe-owned while reducing drift from the shared helper owner surface.

### 2) Gallery/docs now spell out the owner split more explicitly

`apps/fret-ui-gallery/src/ui/pages/combobox.rs` now states that the responsive combobox lane stays
recipe-owned even though shell classification delegates to the shared helper.

That keeps the public-surface story clear:

- helper owner: `fret-ui-kit` / `fret::adaptive`
- recipe owner: `fret-ui-shadcn::Combobox`

### 3) A focused source gate now pins the wrapper boundary

`apps/fret-ui-gallery/tests/device_shell_recipe_wrapper_surface.rs` now keeps three boundaries
visible:

- `Combobox` remains the only current recipe-owned wrapper exemplar,
- `Date Picker` and `Breadcrumb` stay on the shared helper lane,
- `Dialog` and `Sidebar` stay outside wrapper growth.

### 4) Existing behavior gates remain intact

The lane did not change responsive combobox behavior.

`ecosystem/fret-ui-shadcn/tests/combobox_responsive_breakpoint.rs` remains part of the gate set so
the responsive popover-vs-drawer behavior is still proven after the internal owner alignment.

## Gates that define the shipped surface

- `cargo nextest run -p fret-ui-gallery --test device_shell_recipe_wrapper_surface --test device_shell_strategy_surface --test combobox_docs_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --test combobox_responsive_breakpoint --no-fail-fast`
- `git diff --check`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/device-shell-recipe-wrapper-surface-v1/WORKSTREAM.json > /dev/null`

## Follow-on policy

Do not reopen this lane for:

- generic wrapper growth without a second stable family-specific candidate,
- panel/container adaptive work,
- default-prelude widening,
- or collapsing dialog/sidebar proof surfaces into recipe-owned wrappers.

If future work is needed, open a narrower follow-on such as:

1. a second family-specific wrapper with independent proof that policy repetition is real,
2. richer device-shell policy inside an existing wrapper beyond width-only switching,
3. or a docs/public-surface follow-on if the current `Combobox` exemplar stops being the best
   teaching surface.
