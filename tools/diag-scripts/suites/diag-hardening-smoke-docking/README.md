# diag-hardening-smoke-docking

This suite is a small post-merge smoke for docking (multi-viewport + multi-window tear-off flows),
including in-window floating title-bar drag behaviors.

Recommended run (native):

```sh
cargo run -p fretboard -- diag suite diag-hardening-smoke-docking --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Notes:

- Requires desktop runners with `diag.multi_window` capability.
- Some scripts rely on reliable window hover detection (`ui.window_hover_detection=Reliable`).
- `diag suite` runs `diag lint` for captured bundles by default and fails on error-level findings.
