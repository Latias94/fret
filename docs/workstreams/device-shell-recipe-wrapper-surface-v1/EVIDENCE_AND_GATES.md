# Device-Shell Recipe Wrapper Surface v1 — Evidence and Gates

Status: Closed closeout reference
Last updated: 2026-04-11

## Smallest current repro

Use this focused gate set to verify the shipped wrapper-vs-helper boundary:

```bash
cargo nextest run -p fret-ui-gallery --test device_shell_recipe_wrapper_surface --test device_shell_strategy_surface --test combobox_docs_surface --no-fail-fast
cargo nextest run -p fret-ui-shadcn --test combobox_responsive_breakpoint --no-fail-fast
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/device-shell-recipe-wrapper-surface-v1/WORKSTREAM.json > /dev/null
```

What this proves now:

- `Combobox` remains the current recipe-owned device-shell wrapper exemplar.
- That wrapper now delegates binary desktop/mobile classification to the shared helper owner in
  `fret-ui-kit`.
- `Date Picker` and `Breadcrumb` still use the explicit shared helper lane in app/gallery code
  instead of growing new recipe wrappers.
- `Dialog` vs `Drawer` and `SidebarProvider::is_mobile(...)` remain explicit non-wrapper
  boundaries.

## Current evidence set

- `docs/workstreams/device-shell-recipe-wrapper-surface-v1/M0_BASELINE_AUDIT_2026-04-11.md`
  - assumptions-first baseline and the smallest-slice decision for this lane.
- `docs/workstreams/device-shell-recipe-wrapper-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - final verdict, shipped scope, and reopen policy.
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - prior closed owner-split lane that intentionally deferred wrapper growth.
- `docs/workstreams/device-shell-adaptive-facade-promotion-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - prior closed facade lane that intentionally deferred wrapper growth again after promotion.
- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
  - target rule that recipe wrappers may stay family-specific while app-local/device-shell proof
  surfaces remain explicit.
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
  - contract background for device-shell vs container adaptive naming.
- `ecosystem/fret-ui-kit/src/adaptive.rs`
  - shared `DeviceShellSwitchPolicy` and `device_shell_mode(...)` owner surface.
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
  - existing wrapper exemplar plus internal delegation to the shared helper owner.
- `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
  - docs-path note that keeps the responsive combobox lane recipe-owned and explicit.
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
  - app-local helper consumer for `Popover` vs `Drawer`.
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
  - app-local helper consumer for `DropdownMenu` vs `Drawer`.
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
  - explicit docs/proof pairing outside wrapper growth.
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
  - app-shell boundary for `SidebarProvider::is_mobile(...)`.
- `apps/fret-ui-gallery/tests/device_shell_recipe_wrapper_surface.rs`
  - focused source test for the current wrapper-vs-helper-vs-app-shell split.
- `ecosystem/fret-ui-shadcn/tests/combobox_responsive_breakpoint.rs`
  - behavior gate that keeps the shipped responsive combobox behavior intact.

## Shipped gate set

### Source evidence gate

```bash
cargo nextest run -p fret-ui-gallery --test device_shell_recipe_wrapper_surface --test device_shell_strategy_surface --test combobox_docs_surface --no-fail-fast
```

### Behavior gate

```bash
cargo nextest run -p fret-ui-shadcn --test combobox_responsive_breakpoint --no-fail-fast
```

### Diff and doc hygiene

```bash
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/device-shell-recipe-wrapper-surface-v1/WORKSTREAM.json > /dev/null
```

## Follow-on policy

This lane is now closed.

Future work should only reopen as a narrower follow-on if fresh evidence appears for:

- a second stable family-specific recipe wrapper candidate above `device_shell_*`,
- richer device-shell policy inside recipe wrappers beyond width-only switching,
- or a docs/public-surface mismatch that shows `Combobox` is no longer the right exemplar.
