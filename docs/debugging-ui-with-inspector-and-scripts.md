# Debugging UI (Radix/Shadcn) with Inspect + Scripts

This document describes a GPUI/Zed-inspired debugging workflow for Fret UIs, with a focus on:

- **Fast diagnosis** (what is under the pointer / why input is blocked / why focus moved)
- **Deterministic repro** (selector-driven scripted actions)
- **AI-friendly artifacts** (`bundle.json`, pick results, and stable selectors)

Scope note:

- This file focuses on the **interactive inspect workflow** (picker UX + shortcuts).
- For the **bundle format + script harness** details (dump workflow, script schema v1, CI-friendly runs), see:
  `docs/ui-diagnostics-and-scripted-tests.md`.

## Quick start (UI Gallery)

1. Start the demo app:

   - `cargo run -p fret-ui-gallery`

2. Enable continuous inspect:

   - `cargo run -p fretboard -- diag inspect on`

3. Hover + click in the app window:

   - Hover shows candidate outlines.
   - Click writes `target/fret-diag/pick.result.json`.

4. Run the baseline scripted suite:

   - `cargo run -p fretboard -- diag suite ui-gallery`

5. (Perf triage) run the perf harness and print the slowest frames:

   - `cargo run -p fretboard -- diag perf ui-gallery --sort time`

## Inspect workflow (GPUI/Zed-style)

Inspect mode is designed to answer “what is this?” and “why did that happen?” quickly.

In-app shortcuts while inspect mode is active:

- `Esc`: exit inspect (writes `inspect.json` + touches `inspect.touch`)
- `Ctrl+C` / `Cmd+C`: copy the best selector JSON
- `Ctrl+Shift+C` / `Cmd+Shift+C`: copy `selector + focus + path` details
- `F`: lock selection to the current semantics focus
- `L`: lock/unlock selection (freeze hover)
- `Alt+Up` / `Alt+Down`: walk the semantics parent chain (and “back to child”)

### What to look at

- **Outline label**: `role`, `node`, optional `element`, optional `test_id`, optional `label`
- **Panel lines**:
  - `focus: ...` shows the current inspect selection summary
  - `path: ...` shows a compact parent chain (prefers `test_id` over labels)
  - `selector: ...` shows the current best selector JSON

## Radix-aligned primitives: how to debug common problems

Most “it feels broken” bugs in editor-grade UIs come from a small set of mechanisms:

### 1) Overlay routing / “why are clicks blocked?”

Checklist:

- Inspect `debug.layers_in_paint_order` for:
  - which roots are visible,
  - which roots are hit-testable,
  - which root is a **barrier** (`blocks_underlay_input=true`).
- Inspect `debug.semantics.barrier_root` to confirm which subtree is allowed to receive input.
- Use inspect to pick the overlay root, then walk `Alt+Up` to see where the barrier comes from.

Typical causes:

- A dismissible layer that stayed interactive after it should have been closed.
- A presence/motion animation that kept `interactive=true` longer than expected.

### 2) Focus trap / restore / “why did focus jump?”

Checklist:

- Inspect `debug.semantics.focus` and the selected node’s `actions.focus`.
- If a dialog closes, verify focus returns to the trigger:
  - Use `test_id` on the trigger and the close button.
  - Add a scripted repro that asserts focus after closing.

Typical causes:

- A focus scope not restoring focus to the correct element.
- A composite widget using `active_descendant` incorrectly (focus stays on controller).

### 3) Outside-press dismissal / “why doesn’t it close on click outside?”

Checklist:

- Verify the overlay root is receiving pointer events.
- Ensure the “outside press” logic runs in the correct order relative to nested overlays.
- Use inspect to confirm you are clicking outside the content bounds (not outside the visual clip).

## Shadcn recipes: practical authoring rules for debuggability

### Prefer `test_id` for automation stability

For components intended to be scripted (menus, dialogs, dropdowns), always attach `test_id` to:

- triggers,
- primary actions (close/confirm),
- important menu items.

This makes scripts resilient against:

- localization,
- label tweaks,
- path changes.

### Tag non-interactive containers via `Semantics`

If you need a stable handle for a container (panel root, section wrapper), wrap it with:

- `SemanticsProps { role: SemanticsRole::Generic, test_id: Some(\"...\") , .. }`

This keeps `test_id` strictly in the semantics/debug surface (not styling/policy).

## Scripted behavior tests

Fret’s scripted actions are *selector-driven* and run inside the app via file triggers.

### Running scripts

- Run one script:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json`
- Run one script and auto-pack a shareable zip (bundle + `_root/` artifacts):
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json --pack --include-all`
- Run the baseline suite:
  - `cargo run -p fretboard -- diag suite ui-gallery`

### Keeping scripts robust

- Prefer `{"kind":"test_id","id":"..."}` whenever possible.
- Avoid relying on `label` when localization or wording is expected to change.
- Use `wait_until`/`assert` instead of fixed delays when testing overlay open/close and focus restore.
- When UI structure changes, update selectors using:
  - `cargo run -p fretboard -- diag pick-apply <script> --ptr <json-pointer>`

### Visual overlay debugging (optional screenshots)

If a bug is “bounds look right but pixels look wrong”, enable GPU-readback screenshots and author scripts with
`capture_screenshot`. The bundle viewer can then render screenshots as a background overlay (auto-matched by
`manifest.json`):

- Enable screenshots: `FRET_DIAG_SCREENSHOTS=1` (see `docs/ui-diagnostics-and-scripted-tests.md`)
- Offline viewer: `tools/fret-bundle-viewer`

## Accessibility (a11y) notes

Fret’s semantics snapshot is shared by:

- automation/diagnostics (selectors, inspector, scripts),
- accessibility bridges (platform a11y).

`test_id` is **debug/test-only**:

- it is stored separately from the accessible label/name,
- it must not be mapped into platform a11y name fields by default.

This means `test_id` should not “fight” a11y; it is designed to coexist safely.
