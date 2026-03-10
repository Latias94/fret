# Examples

This folder is a **GitHub-friendly portal** to Fret’s runnable examples, modeled after Bevy’s
top-level `examples/` index.

This repo intentionally borrows **Bevy-style discoverability**, but not Bevy?s single-package root
`cargo run --example ...` execution model. The top-level `examples/` directory is a portal only:
runnable lessons live in `apps/fret-cookbook/examples/`, component catalog coverage lives in
`apps/fret-ui-gallery/`, and platform-specific demos remain owned by their app crates.

Note: this repository root is a Cargo workspace (not a package), so `cargo run --example ...` does
not apply at the workspace root. Use the commands below.

Canonical docs index: [`docs/examples/README.md`](../docs/examples/README.md).

## The bare minimum (recommended)

```bash
cargo run -p fretboard -- dev native --example hello
cargo run -p fretboard -- dev native --example simple_todo
```

## Cookbook (lesson-shaped examples)

Cookbook examples live in `apps/fret-cookbook/examples/`.

- Start here: [`apps/fret-cookbook/README.md`](../apps/fret-cookbook/README.md)
- Full index (Bevy-style tables + feature gates + diag suites): [`apps/fret-cookbook/EXAMPLES.md`](../apps/fret-cookbook/EXAMPLES.md)
- List all cookbook examples (shows feature hints for Labs): `cargo run -p fretboard -- list cookbook-examples --all`

## UI Gallery (component catalog + conformance)

```bash
cargo run -p fret-ui-gallery
```

More details: [`apps/fret-ui-gallery/README.md`](../apps/fret-ui-gallery/README.md)

## Templates (generate a new app)

See the “boring ladder” and generators:

- `simple-todo` = the minimal starter path (view runtime + typed actions + keyed lists).
- `todo` = the fuller best-practice baseline once you want selectors + queries.
- [`docs/examples/README.md`](../docs/examples/README.md)
