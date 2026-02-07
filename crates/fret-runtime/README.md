# `fret-runtime`

Portable runtime contracts and value types shared across the Fret workspace.

This crate sits between:

- the **portable vocabulary** (`fret-core`),
- the **UI runtime substrate** (`fret-ui`),
- the **app runtime** (`fret-app`),
- and the **backend glue** (launchers/runners that drain effects and drive scheduling).

It must remain backend-agnostic and avoid forcing a specific async runtime.

## Async policy (v1)

The contract surface in this crate must not require Tokio (or any specific executor).
Async work is **app-owned** and the runtime boundary communicates via explicit messages/effects.

See `docs/workstreams/bottom-up-fearless-refactor-v1.md` for the program-level policy.

## Module ownership map (v1)

- `effect` — portable effect vocabulary (what the UI/app requests; runners drain/execute)
- `execution` — dispatcher/inbox draining traits and types (runner-owned scheduling boundary)
- `model` — `Model<T>` and model store contracts (app-owned data; UI observes)
- `command`, `commands` — command IDs, registries, metadata (keymap/menu integration)
- `keymap`, `when_expr` — keybinding formats and `when` gating expression vocabulary
- `input` — input dispatch phases/default actions and portable shortcuts vocabulary
- `capabilities` — capability signals and execution capabilities (portability modeling)
- `ui_host` — host trait surface (globals/models/commands/effects/time/drag)
- `menu` — portable menu model + file formats (roles/system menus)
- `window_*` — window-scoped services and snapshots used by app + UI (metrics, gating, arbitration, text snapshots)
- `platform_*` — portability-facing platform query contracts (completion, text input queries)
- `font_*`, `text_interaction_settings` — font catalog bootstrap/cache and text interaction defaults
- `docking_settings`, `drag`, `interaction_diagnostics`, `time` — small supporting modules

If you need to add something new:

1. If it is an OS binding, it does not belong here (put it in a backend crate).
2. If it is UI policy, it does not belong here (put it in ecosystem crates).
3. If it is a hard-to-change contract, link an ADR and add a regression gate.

