# ImUi Label Identity Ergonomics v1 - Closeout Audit - 2026-04-28

Status: closed

## Verdict

This lane is closed. Fret IMUI now has a private policy-layer parser for Dear ImGui-style label
identity grammar and applies it consistently across the admitted label-bearing control helpers.

No `fret-ui` runtime identity contract, diagnostics `test_id` contract, accessibility contract, or
localization policy was widened.

## Adopted Surface

Controls without an explicit string ID now key their helper-owned subtree by parsed label identity:

- button, small button, and action button
- selectable rows and multi-select rows through selectable delegation
- menu item rows, including checkbox/radio/action/submenu item paths
- checkbox
- radio
- switch
- slider

Controls with an explicit string ID keep that ID as the identity owner and use only the parsed
visible label for rendering and default accessibility labels:

- combo and combo-model triggers
- menu and submenu triggers
- tab triggers
- collapsing headers
- tree nodes

Label-only helpers with no item identity use only the parsed visible label:

- `separator_text`

## Deferred Scope

Deferred by design:

- runtime ID-stack debugging and ID conflict diagnostics
- `test_id` inference from labels
- localization policy for labels that contain `##` / `###`
- generic `ui.text` rendering, which remains literal text
- table header / column display-name policy
- Linux compositor or multi-window acceptance, which remains owned by docking / multi-window lanes

## Gate Evidence

- `cargo nextest run -p fret-ui-kit --features imui imui_label_identity --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 1`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`

The `fret-ui-kit` parser gate initially reproduced a Windows/MSVC `link.exe` LNK1120 failure with
incremental test artifacts. The lane closed by setting `incremental = false` for
`[profile.test.package.fret-ui-kit]`, after which the standard gate command passed without extra
environment variables.
