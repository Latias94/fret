from pathlib import Path
from argparse import Namespace
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
        cmd = gate._diag_perf_prefix(Path("target/release/fretboard-dev.exe"), "ui-code-editor-resize-probes")

        self.assertEqual("target/release/fretboard-dev.exe", cmd[0].replace("\\", "/"))
        self.assertEqual(["diag", "perf", "ui-code-editor-resize-probes"], cmd[1:])

    def test_default_launch_cmd_is_inspectable_cargo_gallery_full(self) -> None:
        self.assertEqual(
            [
                "cargo",
                "run",
                "-p",
                "fret-ui-gallery",
                "--release",
                "--features",
                "gallery-full",
            ],
            gate._default_launch_cmd(),
        )

    def test_launch_cmd_from_args_prefers_explicit_tokens(self) -> None:
        launch_cmd, launch_bin_path = gate._launch_cmd_from_args(
            Path("F:/repo"),
            Namespace(launch_cmd="cargo run -p fret-ui-gallery", launch_bin="target/release/fret-ui-gallery"),
        )

        self.assertEqual(["cargo", "run", "-p", "fret-ui-gallery"], launch_cmd)
        self.assertIsNone(launch_bin_path)

    def test_launch_cmd_from_args_can_preserve_app_separator(self) -> None:
        launch_cmd, launch_bin_path = gate._launch_cmd_from_args(
            Path("F:/repo"),
            Namespace(launch_cmd="cargo run -p fret-ui-gallery --release -- --help", launch_bin=""),
        )

        self.assertEqual(["cargo", "run", "-p", "fret-ui-gallery", "--release", "--", "--help"], launch_cmd)
        self.assertIsNone(launch_bin_path)

    def test_launch_cmd_from_args_preserves_legacy_launch_bin(self) -> None:
        launch_cmd, launch_bin_path = gate._launch_cmd_from_args(
            Path("F:/repo"),
            Namespace(launch_cmd=None, launch_bin="target/release/fret-ui-gallery.exe"),
        )

        self.assertEqual(["F:/repo/target/release/fret-ui-gallery.exe"], [token.replace("\\", "/") for token in launch_cmd])
        self.assertEqual("F:/repo/target/release/fret-ui-gallery.exe", str(launch_bin_path).replace("\\", "/"))


if __name__ == "__main__":
    unittest.main()
