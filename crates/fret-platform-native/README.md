# `fret-platform-native`

Native (non-wasm) implementations of `fret-platform` contracts.

This crate provides desktop implementations for:

- clipboard (`arboard`)
- file dialogs (`rfd`)
- open-url (`webbrowser`)
- external drop payload reading via real filesystem paths

## Module ownership map

- `src/clipboard.rs`: `NativeClipboard` (and `DesktopClipboard` alias).
- `src/external_drop.rs`: `NativeExternalDrop` token/payload storage and reading.
- `src/file_dialog.rs`: `NativeFileDialog` selection storage and reading.
- `src/open_url.rs`: `NativeOpenUrl`.

## Public surface

Prefer importing the primary implementation types from `fret_platform_native`’s re-exports in
`src/lib.rs`.

## Refactor gates

- Formatting: `cargo fmt`
- Build/test: `cargo nextest run -p fret-platform-native`

