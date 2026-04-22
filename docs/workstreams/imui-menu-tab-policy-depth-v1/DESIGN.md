# ImUi Menu/Tab Policy Depth v1

Status: active execution lane
Last updated: 2026-04-22

Status note (2026-04-22): the first admitted generic IMUI policy-depth floor has now landed:
top-level menubar hover-switch plus submenu hover-open / sibling hover-switch with an end-to-end
enforced grace corridor, alongside the same-frame reverse-direction top-level focus handoff fix.
The remaining scope is narrower than the original baseline audit: editor-grade tab overflow /
scroll / reorder / close stays in `fret-workspace`, and outer-scope mnemonic / roving posture now
has an explicit shell-owned verdict, so the only open generic IMUI question is whether any richer
submenu-intent tuning beyond the current enforced corridor should remain generic.

Related:

- `WORKSTREAM.json`
- `M0_BASELINE_AUDIT_2026-04-21.md`
- `M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `M2_TAB_OWNER_VERDICT_2026-04-22.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

## Why this lane exists

The helper-owned menu/submenu/tab outward-response question is already closed:

- `begin_menu[_with_options]` / `begin_submenu[_with_options]` now return
  `DisclosureResponse`,
- `tab_bar[_with_options]` now returns `TabBarResponse`,
- and the canonical naming cleanup already removed the duplicate `*_response*` alias layer.

What remains open is a different problem:

- richer submenu intent tuning beyond the current enforced corridor.

This lane exists so that follow-on work does not reopen:

- response-surface naming,
- `ResponseExt` lifecycle vocabulary,
- key-owner surface work,
- collection/pane proof breadth,
- workbench shell helper promotion,
- or runtime widening.

## Problem statement

Fret's immediate menu/tab family already covers:

- click-open top-level menus,
- nested submenus,
- simple tab selection and panel switching,
- and helper-owned outward trigger responses.

But editor-grade parity still lacks a narrow owner for:

- richer submenu grace / intent behavior beyond the current enforced corridor.

## Owner split

In scope for this lane:

- `ecosystem/fret-ui-kit::imui`
- `ecosystem/fret-imui` focused tests
- first-party IMUI proof surfaces that exercise generic menu/tab behavior

Out of scope for this lane:

- `crates/fret-ui`
- `fret-authoring::Response`
- global shortcut/key-owner arbitration
- workbench-shell-specific tabstrip product helpers in `fret-workspace`
- runner/backend multi-window behavior

## Initial design posture

The lane starts from one explicit assumption:

- not every Dear ImGui or workbench-shell tab affordance should become a generic IMUI helper.

Current working posture:

1. Generic menubar/submenu hover-switch depth is the most credible first IMUI-owned slice.
2. Editor-grade tab overflow / reorder / close remains a `fret-workspace` shell concern until a
   different first-party consumer proves the policy is generic.
3. Roving focus / mnemonic policy should not be widened casually; it needs explicit evidence that
   the generic IMUI family, not shell/product owners, benefits from the added contract.

## First landable target

Before broadening surface area, this lane should freeze one smallest executable slice.

The current preferred order is:

1. audit and, if justified, land top-level menubar hover-switch plus submenu hover-switch/basic
   grace corridor;
2. keep richer editor-grade tab policy in `fret-workspace` unless a new first-party consumer
   proves it belongs in generic IMUI;
3. re-evaluate whether any generic roving/mnemonic policy belongs in the same family.

## Non-goals

This lane must not be used to justify:

- a new immediate runtime,
- a new global key-owner shortcut registration seam,
- a generic shell/workspace tabstrip abstraction,
- or broad helper growth just because Dear ImGui exposes more flags.
