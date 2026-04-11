# Device-Shell Strategy Surface v1 — Target Interface State

Status: target state for M1 device-shell strategy freeze
Last updated: 2026-04-11

Companion docs:

- `DESIGN.md`
- `M0_BRANCH_SITE_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `M1_CONTRACT_FREEZE_2026-04-11.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This document records the intended end state for the higher-level device-shell strategy surface.

It answers four concrete questions:

1. which device-shell nouns ordinary app authors should learn,
2. where binary desktop/mobile branch selection should live,
3. which surfaces should remain recipe-owned or app-local,
4. which current-looking shortcuts should be rejected for new APIs.

## 1. Public Surface Tiers

| Tier | Intended audience | Canonical import lane | What it owns |
| --- | --- | --- | --- |
| Low-level reads | advanced app code, diagnostics-heavy examples | `fret::env::{...}` | raw viewport/device queries, safe area, occlusion, pointer capability |
| Shared classification | app authors and reusable helpers | `fret::adaptive::{...}` | `DeviceAdaptiveClass`, `DeviceAdaptivePolicy`, `device_adaptive_class`, `device_adaptive_snapshot` |
| Shared binary strategy | reusable component/app helpers | target: `fret_ui_kit::adaptive::{...}` first | explicit desktop/mobile branch choice above raw reads |
| Recipe wrappers | design-system/component crates | `fret-ui-shadcn` and future ecosystem crates | source-aligned family wrappers such as combobox or future overlay-shell helpers |
| App-local explicit seams | app/gallery code | explicit local code | one-off pairings that are still evidence or product-specific composition |

## 2. Target Authoring Rule

### 2.1 Raw reads stay explicit

`fret::env` remains the only lane for raw viewport/device reads.

Shared strategy must not teach ordinary app authors to start from:

- `viewport_width_at_least(...)`,
- pointer capability probes,
- or direct environment width checks

when the real question is simply "desktop shell or mobile shell?"

### 2.2 Shared device-shell strategy belongs above classification, not inside it

The target split is:

- `fret::adaptive`
  - classification nouns only
  - "what kind of device-shell environment is this?"
- `fret-ui-kit`
  - branch-selection helpers
  - "given explicit policy, which desktop/mobile shell branch should I render?"

This means the new shared surface should not first land as another top-level `fret::adaptive`
export.

The first durable owner is `fret-ui-kit`.

### 2.3 The shared helper should be binary and explicit

The target shared helper is a binary device-shell switcher, not a new generic responsive manager.

Target concept inventory:

| Concept | Target role | Notes |
| --- | --- | --- |
| `DeviceShellMode` | binary branch result | explicit `Desktop` vs `Mobile` choice |
| `DeviceShellSwitchPolicy` | branch-selection policy | wraps thresholds / pointer-capability bias if needed |
| `device_shell_mode(...)` | pure selector | derives `DeviceShellMode` above raw reads |
| `device_shell_switch(...)` or equivalent | authoring helper | chooses one of two explicit branches without widening generic children APIs |

The exact final names can still be tuned during landing, but the ownership rule is fixed:

- explicit `device_shell_*` naming,
- binary desktop/mobile branch result,
- and helper ownership in `fret-ui-kit` first.

### 2.4 Shared strategy should not own recipe policy

The shared helper may choose a branch.
It should not also own:

- popover placement policy,
- drawer snap-point policy,
- dialog content layout,
- app-shell sidebar state models,
- or editor rail ownership.

Those remain recipe-owned or app-owned.

## 3. Naming Rules for New Public APIs

### 3.1 New shared APIs must say `device_shell`

For new shared helper names, prefer:

- `DeviceShellMode`
- `DeviceShellSwitchPolicy`
- `device_shell_mode(...)`
- `device_shell_switch(...)`

Reject new shared names such as:

- `responsive_mode`
- `responsive_branch`
- `adaptive_overlay`
- bare `responsive(bool)`

### 3.2 Recipe wrappers may stay family-specific

Recipe-owned wrappers may keep family nouns in their API so long as device-shell ownership is still
visible at the boundary.

Current acceptable exemplar:

- `Combobox::device_shell_responsive(...)`
- `Combobox::device_shell_md_breakpoint(...)`

That is acceptable because:

- the branch is explicitly device-shell-driven,
- and the recipe still owns the resulting `Popover` vs `Drawer` policy.

### 3.3 `Sidebar` remains app-shell vocabulary

`SidebarProvider::is_mobile(...)` and `is_mobile_breakpoint(...)` remain acceptable because they
are already clearly app-shell/device-shell controls.

They should not be renamed into a generic overlay or panel-adaptive vocabulary.

## 4. Family Classification

### 4.1 Best first shared-helper candidate

These current surfaces show the most repeated branch shape:

- `Popover` vs `Drawer`
- `DropdownMenu` vs `Drawer`

Target rule:

- prefer one shared binary device-shell switcher beneath these pairings before inventing many
  family-specific wrappers

### 4.2 Keep explicit for now

These should remain explicit until repetition proves a better wrapper:

- `Dialog` vs `Drawer`
- proof/demo pairings that intentionally show both branches side by side

Target rule:

- keep the pairing explicit if the surface is still primarily a docs/proof exemplar rather than a
  repeated reusable authoring need

### 4.3 Keep provider/app-shell-owned

These stay outside the shared helper target:

- `SidebarProvider::is_mobile(...)`
- `SidebarProvider::is_mobile_breakpoint(...)`
- `open_mobile`
- `width_mobile`

Target rule:

- sidebar device inference remains an app-shell/provider surface

### 4.4 Keep recipe-internal

These stay out of the shared helper target:

- `Dialog` internal `sm:` alignment logic
- `Sheet` internal width/max-size breakpoints
- `AlertDialog` internal layout/chrome breakpoints

Target rule:

- internal parity logic is not itself evidence for a new shared public strategy surface

## 5. `children(...)` Boundary

This lane does not justify broader generic `children(...)` growth.

Target rule:

- a shared device-shell helper should accept two explicit branches or equivalent typed inputs,
- it should not reopen generic slot/root-children growth across overlay families,
- and recipe wrappers should continue to own their family-specific content builders.

## 6. App-Facing Export Rule

Current target:

- keep shared device-shell strategy crate-local to `fret-ui-kit` first
- continue exposing only classification nouns on `fret::adaptive`

Promotion rule:

- only re-export a device-shell strategy helper from `fret` after one landed helper/wrapper proves
  stable across at least two real consumers

## 7. Rejected Interface State

The target state explicitly rejects:

- a new generic `responsive(bool)` shared helper,
- device-shell strategy living in `crates/fret-ui`,
- automatic export of shared device-shell helpers from the default app prelude,
- turning `Sidebar` into the default shared strategy template,
- and using panel/container vocabulary for desktop/mobile shell branching.

## 8. Minimum Proof Before First Extraction

Before landing the first shared helper or wrapper, keep these proofs green:

- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
- the targeted `ui_authoring_surface_default_app` assertions already used by this lane

This keeps the M1 freeze tied to the existing branch-site evidence instead of reopening another
broad adaptive audit.
