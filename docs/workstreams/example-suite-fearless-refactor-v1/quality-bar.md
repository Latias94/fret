# Example Suite v1 — Official Example Quality Bar

This appendix defines the “official example” acceptance bar.

The purpose is to make examples:

- teachable (small, focused, copy/paste friendly),
- discoverable (stable IDs),
- regression-ready (gates),
- maintainable (clear ownership, no drift).

## Definition: “official” vs “maintainer” examples

- **Official example**: user-facing, referenced by onboarding docs, and listed first by tooling.
- **Maintainer example**: still runnable, but intended for stress/regression/dev; hidden by default.

## Required metadata (doc header)

Every official example must have a short header (in its README or file header) that includes:

- **ID**: stable string used in commands and scripts.
- **Goal**: 1–2 sentences.
- **Teaches**: 1–3 bullet points.
- **Run**: exact command(s), including web/native variants and required env vars.
- **What to edit**: which file(s) are intended user entry points.
- **Gates**: which diag script(s) / tests / perf baselines apply.
- **Owner**: which crate/layer “owns” the behavior (ecosystem vs mechanism).

## Stable IDs and naming rules

### Example ID

Rules:

- snake_case or kebab-case, but choose one style and stick to it (recommend snake_case in Rust surfaces).
- no platform suffix in the canonical ID (use tiering metadata instead).
- avoid “demo” in the ID when it is meant to be user-facing (“demo” implies throwaway).

### `test_id` conventions

`test_id` is part of the tooling contract (diag scripts, UI automation, drift detection).

Rules:

- `test_id` strings must be stable and intention-revealing.
- Use a consistent prefix based on the example ID:
  - `todo.add`
  - `workbench.command_palette.input`
  - `shader_lab.customv3.toggle`
- Prefer dot-separated segments:
  - `<example>.<surface>.<control>.<action>`
- Avoid embedding dynamic values unless required; if dynamic, keep it normalized:
  - `todo.item.<id>.remove` is acceptable when `<id>` is the stable model key.

## Gates (minimum required)

Every official example must have at least one gate:

- a `fretboard-dev diag run ...` script that:
  - performs 1–3 high-signal actions,
  - captures at least one screenshot or asserts a simple invariant, or
- a small Rust test asserting a contract outcome, or
- a perf baseline if the example exists primarily to protect performance.

## Dependency hygiene (keep examples teachable)

Official cookbook examples should:

- default to `fret` + ecosystem surfaces (`fret-ui-shadcn`, `fret-ui-kit`),
- avoid pulling backend crates directly (`winit`, `wgpu`, `fret-launch`) unless the example *is explicitly about that boundary*,
- keep the “what to copy” surface small (one file if possible).

## Promotion checklist (when adding a new official example)

- [ ] ID chosen and recorded in the catalog.
- [ ] Doc header present and correct.
- [ ] `test_id` stamps added to primary controls.
- [ ] At least one gate exists and is linked.
- [ ] Web tier set (W0/W1/W2) with explicit notes.
- [ ] Owner identified (ecosystem/mechanism) and linked to relevant ADR/workstream if needed.

