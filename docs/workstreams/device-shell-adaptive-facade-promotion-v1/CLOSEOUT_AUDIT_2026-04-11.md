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
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`

## Verdict

This lane is now closed.

The shipped `device_shell_*` helper is now promoted onto the explicit `fret::adaptive::{...}` lane
for app-facing code, while the real owner remains `fret-ui-kit` and the default preludes stay
unchanged.

## What shipped

### 1) `fret::adaptive` now re-exports the explicit device-shell strategy surface

`ecosystem/fret/src/lib.rs` now re-exports:

- `DeviceShellMode`
- `DeviceShellSwitchPolicy`
- `device_shell_mode(...)`
- `device_shell_switch(...)`

alongside the earlier adaptive classification nouns.

### 2) App-facing proof surfaces now use the promoted lane

The gallery's app-facing device-shell snippets now import from `fret::adaptive::{...}` instead of
reaching into `fret_ui_kit::adaptive` directly:

- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`

That keeps the explicit app-facing lane coherent with the shipped promotion.

### 3) Default preludes did not widen

The lane explicitly kept:

- `fret::app::prelude::*`
- `fret::component::prelude::*`

free of the adaptive strategy nouns.

So promotion happened on the explicit lane only, not on the default first-contact surface.

## Gates that define the shipped surface

- `cargo nextest run -p fret root_surface_exposes_explicit_adaptive_module app_prelude_stays_explicit_instead_of_reexporting_legacy_surface app_prelude_omits_low_level_mechanism_types component_prelude_is_curated_for_reusable_component_authors usage_docs_expose_curated_component_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test device_shell_strategy_surface --no-fail-fast`
- `git diff --check`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/device-shell-adaptive-facade-promotion-v1/WORKSTREAM.json > /dev/null`

## Follow-on policy

Do not reopen this lane for:

- helper-shape redesign,
- recipe-owned wrapper growth,
- default prelude expansion,
- or broader panel/container adaptive work.

If future work is needed, open a narrower follow-on such as:

1. a recipe-owned wrapper lane above the promoted helper,
2. richer device-shell policy beyond width-only switching,
3. or a docs/examples lane that teaches the promoted explicit app-facing import surface more
   broadly.
