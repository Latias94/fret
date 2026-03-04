# Examples

This folder is a **GitHub-friendly portal** to Fret’s runnable examples, modeled after Bevy’s
top-level `examples/` index.

Note: this repository root is a Cargo workspace (not a package), so `cargo run --example ...` does
not apply at the workspace root. Use the commands below.

## The bare minimum (recommended)

```bash
cargo run -p fret-cookbook --example hello
cargo run -p fret-cookbook --example simple_todo
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

- [`docs/examples/README.md`](../docs/examples/README.md)
