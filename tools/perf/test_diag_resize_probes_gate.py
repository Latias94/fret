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


if __name__ == "__main__":
    unittest.main()
