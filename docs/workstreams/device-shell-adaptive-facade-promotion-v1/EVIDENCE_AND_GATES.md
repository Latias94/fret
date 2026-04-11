# Device-Shell Adaptive Facade Promotion v1 — Evidence and Gates

Status: Closed closeout reference
Last updated: 2026-04-11

## Smallest current repro

```bash
cargo nextest run -p fret root_surface_exposes_explicit_adaptive_module app_prelude_stays_explicit_instead_of_reexporting_legacy_surface app_prelude_omits_low_level_mechanism_types component_prelude_is_curated_for_reusable_component_authors usage_docs_expose_curated_component_surface --no-fail-fast
cargo nextest run -p fret-ui-gallery --test device_shell_strategy_surface --no-fail-fast
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

What this should prove:

- `fret::adaptive` explicitly re-exports the shipped `device_shell_*` surface,
- the helpers still stay out of default app/component preludes,
- and app-facing gallery code can use the promoted explicit lane instead of importing from
  `fret_ui_kit::adaptive` directly.

## Current evidence set

- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
  - promotion rule: promote only after one landed helper proves stable across at least two real
    consumers.
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - records why promotion was deferred in the previous lane and why it needs a narrower follow-on.
- `ecosystem/fret-ui-kit/src/adaptive.rs`
  - shipped owner of the helper surface.
- `ecosystem/fret/src/lib.rs`
  - current explicit `fret::adaptive` surface and the default-prelude boundary tests.
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
  - current app-facing `Popover` / `Drawer` proof surface.
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
  - current app-facing `DropdownMenu` / `Drawer` proof surface.
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
  - focused source gate for the current app-facing imports and branch visibility.
- `docs/crate-usage-guide.md`
  - current explicit-lane guidance for `fret::adaptive::{...}`.

## Shipped gate set

### Root facade gate

```bash
cargo nextest run -p fret root_surface_exposes_explicit_adaptive_module app_prelude_stays_explicit_instead_of_reexporting_legacy_surface app_prelude_omits_low_level_mechanism_types component_prelude_is_curated_for_reusable_component_authors usage_docs_expose_curated_component_surface --no-fail-fast
```

### App-facing proof gate

```bash
cargo nextest run -p fret-ui-gallery --test device_shell_strategy_surface --no-fail-fast
```

### Diff hygiene

```bash
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```
