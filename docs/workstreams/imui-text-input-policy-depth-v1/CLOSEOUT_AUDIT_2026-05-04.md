# ImUi Text Input Policy Depth v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane is closed as the shipped first text-input depth package. It should not remain the active
execution surface for later text editing work.

## Shipped Scope

- Runtime text input and textarea read-only mechanisms.
- IMUI `InputTextOptions` / `TextAreaOptions` read-only policy.
- IMUI select-all-on-focus policy implemented in `fret-ui-kit::imui`, not `crates/fret-ui`.
- Runtime and IMUI multiline Tab insertion policy, defaulting to no Tab mutation unless
  `TextAreaOptions::allow_tab_input=true`.
- Explicit-key `ImUi::push_id` identity semantics for stable model-backed `changed()` behavior.
- Cookbook proof through the app-facing `fret::imui` surface.

## Superseding Follow-Ons

The callback-heavy Dear ImGui text-input surface was intentionally split into narrower lanes:

- `docs/workstreams/imui-text-input-history-completion-policy-v1/`
- `docs/workstreams/imui-text-input-filter-policy-v1/`
- `docs/workstreams/imui-text-input-custom-filter-policy-v1/`
- `docs/workstreams/imui-text-input-undo-command-policy-v1/`
- `docs/workstreams/imui-text-input-picker-recipe-v1/`
- `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/`

Those lanes close the command routing, named/custom insertion filtering, app-owned undo/redo command
routing, visible completion/history picker UI, and keyboard navigation pieces that were still open
when this lane was first created.

## Remaining Work

Do not reopen this lane for new behavior. Start narrower follow-ons for:

- editor-owned completion/history ranking and persistence policy,
- active-descendant accessibility wiring for generic IMUI picker recipes,
- numeric scalar text-edit fallback,
- deeper multiline wrapping/no-horizontal-scroll behavior,
- or fixture-driven decomposition of the large IMUI text test surface.

## Evidence

- `crates/fret-ui/src/text/input/widget.rs`
- `crates/fret-ui/src/text/area/widget.rs`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`

## Gates

Canonical gate set remains in `EVIDENCE_AND_GATES.md`. The final closeout refresh also requires:

```bash
python -m json.tool docs/workstreams/imui-text-input-policy-depth-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
