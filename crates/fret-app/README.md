# `fret-app`

Application runtime glue for Fret apps.

This crate is the default “app runtime” layer: it wires together models, commands, config files,
settings, menu/menu bar integration, and effect draining into an ergonomic surface for first-party
apps and templates.

It should remain:

- backend-agnostic (no direct `winit`/`wgpu`/`web-sys` dependencies),
- portable across native + web runners (behavioral contracts live in ADRs),
- and free of UI policy (policy-heavy behavior belongs in ecosystem component crates).

## Async policy (v1)

This crate may integrate with async work at the app boundary (background tasks, I/O, network), but
it must not require a specific async runtime in its **public contract surface**. Prefer app-owned
dispatch/execution through `fret-runtime` capabilities and message/effect loops.

## Module ownership map (v1)

- `app` — the `App` runtime entrypoint (models, effects, frame/app lifecycle glue)
- `ui_host` — app-owned `UiHost` glue and host services wiring
- `plugins` — plugin registration and the “default plugins” story for apps/templates
- `settings` — settings file schema + load/apply helpers (`.fret/settings.json`)
- `keymap` — layered keymap loading/merging and default bindings (`keymap.json`)
- `menu`, `menu_bar` — menu model wiring + OS/in-window menubar sync helpers
- `config_files`, `config_watcher` — layered config paths and file change watching
- `dock_layout_file` — persistence helpers for docking layouts
- `drag` — app-owned drag glue used by docking/menu integration
- `core_commands` — baseline commands (app-level “golden path” command registry)
- `app_display_name` — display name plumbing for UI surfaces (e.g. menu role titles)
- `when_expr` — `when` gating helpers at the app layer (complements `fret-runtime`)
- `font_catalog_cache` — app-level font catalog caching glue (bridges runtime and render/text systems)

If you need to add something new:

1. If it is a backend binding, it does not belong here (put it in `fret-launch`/runner/backends).
2. If it is UI interaction policy, it does not belong here (put it in ecosystem crates).
3. If it changes a hard-to-change contract, link an ADR and add a gate.

