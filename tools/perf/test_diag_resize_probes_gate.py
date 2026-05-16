from pathlib import Path
import unittest

import diag_resize_probes_gate as gate


class ResizeProbesGateTests(unittest.TestCase):
    def test_diag_perf_prefix_defaults_to_cargo_run(self) -> None:
        self.assertEqual(
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "fretboard-dev",
                "--",
                "diag",
                "perf",
                "ui-resize-probes",
            ],
            gate._diag_perf_prefix(None, "ui-resize-probes"),
        )

    def test_diag_perf_prefix_accepts_prebuilt_fretboard_binary(self) -> None:
        self.assertEqual(
            [
                "target/release/fretboard-dev.exe",
                "diag",
                "perf",
                "ui-code-editor-resize-probes",
            ],
            gate._diag_perf_prefix(Path("target/release/fretboard-dev.exe"), "ui-code-editor-resize-probes"),
        )

    def test_launch_command_accepts_cargo_run_shape(self) -> None:
        self.assertEqual(
            [
                "cargo",
                "run",
                "-p",
                "fret-ui-gallery",
                "--release",
                "--features",
                "gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness",
                "--",
                "target/release/fret-ui-gallery",
            ],
            gate._launch_command(
                Path("target/release/fret-ui-gallery"),
                "cargo run -p fret-ui-gallery --release --features "
                "gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness -- "
                "target/release/fret-ui-gallery",
            ),
        )

    def test_launch_command_defaults_to_prebuilt_binary(self) -> None:
        self.assertEqual(
            ["target/release/fret-ui-gallery.exe"],
            gate._launch_command(Path("target/release/fret-ui-gallery.exe"), ""),
        )


if __name__ == "__main__":
    unittest.main()
