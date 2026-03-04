# fret-demo (Maintainer harness)

Native demo harness shell for the Fret workspace.

This crate is intentionally **not** the primary onboarding path. If you are new to the repo, prefer:

- Templates ladder: [docs/examples/README.md](../../docs/examples/README.md)
- Cookbook lessons: `cargo run -p fret-cookbook --example hello`
- UI gallery app: `cargo run -p fret-ui-gallery`

This crate serves two maintainer-oriented purposes:

1) A **single binary demo selector** (`fret-demo`) that mirrors the web demo ID vocabulary.
2) A set of **per-demo binaries** under `src/bin/*` so demos can also be run as `--bin <name>` targets
   (useful for per-demo build iteration and isolating regressions).

## Run (native)

List available demo IDs:

```bash
cargo run -p fret-demo -- --list
```

Run a demo by ID (shell mode):

```bash
cargo run -p fret-demo -- todo_demo
cargo run -p fret-demo -- plot_demo
```

Run a demo as a dedicated bin target:

```bash
cargo run -p fret-demo --bin todo_demo
cargo run -p fret-demo --bin components_gallery
```

Recommended: use `fretboard` so the runner flags and profiles are consistent:

```bash
cargo run -p fretboard -- dev native --bin todo_demo
```
