import unittest

import diag_editor_paint_contract_validate as validate


class EditorPaintContractValidateTests(unittest.TestCase):
    def test_default_plan_uses_windows_contract_surfaces(self) -> None:
        plan = validate.build_plan(
            python_bin="python",
            fretboard_bin="target/release/fretboard-dev.exe",
            launch_bin="target/release/fret-ui-gallery.exe",
            out_dir="target/fret-diag/editor-paint-contract-validate-test",
            resize_attempts=3,
            resize_repeat=7,
            typical_repeat=15,
            complex_repeat=7,
            warmup_frames=5,
            skip_preflight=False,
            with_paint_perf=False,
        )

        self.assertEqual(
            ["preflight", "resize-jitter", "typical-autoscroll", "complex-wheel"],
            [step["name"] for step in plan],
        )

        joined = "\n".join(" ".join(step["cmd"]) for step in plan)
        self.assertIn(validate.RESIZE_BASELINE, joined)
        self.assertIn(validate.TYPICAL_BASELINE, joined)
        self.assertIn(validate.COMPLEX_WHEEL_BASELINE, joined)
        self.assertIn("FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0", joined)
        self.assertIn("target/release/fret-ui-gallery.exe", joined)
        self.assertNotIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[2]["cmd"]))
        self.assertNotIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[3]["cmd"]))

    def test_paint_perf_flag_only_marks_non_resize_diag_perf_steps(self) -> None:
        plan = validate.build_plan(
            python_bin="python",
            fretboard_bin="target/release/fretboard-dev.exe",
            launch_bin="target/release/fret-ui-gallery.exe",
            out_dir="target/fret-diag/editor-paint-contract-validate-test",
            resize_attempts=3,
            resize_repeat=7,
            typical_repeat=15,
            complex_repeat=7,
            warmup_frames=5,
            skip_preflight=True,
            with_paint_perf=True,
        )

        self.assertEqual(["resize-jitter", "typical-autoscroll", "complex-wheel"], [step["name"] for step in plan])
        self.assertNotIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[0]["cmd"]))
        self.assertIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[1]["cmd"]))
        self.assertIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[2]["cmd"]))


if __name__ == "__main__":
    unittest.main()
