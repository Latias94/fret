# diag-hardening-smoke-docking

This suite is a small post-merge smoke for docking (multi-viewport + multi-window tear-off flows).

Recommended run (native):

```sh
cargo run -p fretboard -- diag suite diag-hardening-smoke-docking --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Notes:

- Requires desktop runners with `diag.multi_window` capability.
- Some scripts rely on reliable window hover detection (`ui.window_hover_detection=Reliable`).
- This suite may hit known lint findings in the docking demo bundles; use `--no-lint` for smoke verification:
  - `cargo run -p fretboard -- diag suite diag-hardening-smoke-docking --timeout-ms 900000 --no-lint --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`
