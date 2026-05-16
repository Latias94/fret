import json
import io
import sys
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

import diag_editor_paint_contract_closeout as closeout


def _verifier_with_inputs(
    *,
    paint_widget_p95: int,
    canvas_p95: int,
    renderer_text_p95: int,
) -> dict[str, object]:
    return {
        "ok": True,
        "attribution": {
            "steps": {
                "typical-autoscroll": {
                    "decision_inputs": {
                        "paint_widget_p95_us": paint_widget_p95,
                        "renderer_prepare_text_p95_us": renderer_text_p95,
                        "renderer_encode_scene_p95_us": 20,
                        "renderer_upload_p95_us": 30,
                        "code_editor_total_p95_us": 40,
                        "paint_widget_hotspot_summary": {
                            "canvas_exclusive_p95_us": canvas_p95,
                        },
                    }
                }
            }
        },
    }


class EditorPaintContractCloseoutTests(unittest.TestCase):
    def test_closeout_plan_uses_repo_gates(self) -> None:
        plan = closeout.build_plan(
            python_bin="python",
            matrix="docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md",
            workstream_json="docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json",
            skip_diff_check=False,
        )

        self.assertEqual(
            ["perf-baseline-matrix-audit", "workstream-json-valid", "workstream-catalog", "git-diff-check"],
            [step["name"] for step in plan],
        )

        joined = "\n".join(" ".join(step["cmd"]) for step in plan)
        self.assertIn("audit_perf_baselines.py", joined)
        self.assertIn("json.tool", joined)
        self.assertIn("check_workstream_catalog.py", joined)
        self.assertEqual(["git", "diff", "--check"], plan[-1]["cmd"])

    def test_closeout_plan_can_skip_diff_check(self) -> None:
        plan = closeout.build_plan(
            python_bin="python",
            matrix="docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md",
            workstream_json="docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json",
            skip_diff_check=True,
        )

        self.assertEqual(
            ["perf-baseline-matrix-audit", "workstream-json-valid", "workstream-catalog"],
            [step["name"] for step in plan],
        )

    def test_dry_run_does_not_require_synced_artifacts(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            report = root / "closeout-plan.json"
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_closeout.py",
                    str(root / "missing-validation-dir"),
                    "--attribution-dir",
                    str(root / "missing-attribution-dir"),
                    "--dry-run",
                    "--out-report",
                    str(report),
                ],
            ):
                with redirect_stdout(io.StringIO()):
                    rc = closeout.main()

            summary = json.loads(report.read_text(encoding="utf-8"))

        self.assertEqual(0, rc)
        self.assertTrue(summary["ok"])
        self.assertEqual({"skipped": True, "reason": "dry-run"}, summary["verifier"])

    def test_non_dry_run_requires_attribution_dir(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_closeout.py",
                    str(root / "validation-dir"),
                ],
            ):
                with redirect_stderr(io.StringIO()):
                    rc = closeout.main()

        self.assertEqual(2, rc)

    def test_verifier_date_tag_extracts_top_level_closeout_fields(self) -> None:
        verifier = {
            "validation": {"date_tag": "run-a"},
            "attribution": {"date_tag": "run-a-attrib"},
        }

        self.assertEqual("run-a", closeout.verifier_date_tag(verifier, "validation"))
        self.assertEqual("run-a-attrib", closeout.verifier_date_tag(verifier, "attribution"))
        self.assertIsNone(closeout.verifier_date_tag(verifier, "missing"))

    def test_owner_decision_selects_canvas_when_paint_widget_dominates(self) -> None:
        decision = closeout.decide_next_owner(
            _verifier_with_inputs(
                paint_widget_p95=500,
                canvas_p95=320,
                renderer_text_p95=80,
            )
        )

        self.assertEqual("decided", decision["status"])
        self.assertEqual("canvas-paint-replay", decision["owner"])
        self.assertEqual("open-canvas-paint-replay-slice", decision["action"])

    def test_owner_decision_selects_renderer_text_when_it_dominates(self) -> None:
        decision = closeout.decide_next_owner(
            _verifier_with_inputs(
                paint_widget_p95=160,
                canvas_p95=90,
                renderer_text_p95=280,
            )
        )

        self.assertEqual("decided", decision["status"])
        self.assertEqual("renderer-text-prepare", decision["owner"])
        self.assertEqual("open-glyph-text-index-atlas-residency-slice", decision["action"])

    def test_owner_decision_selects_no_code_change_when_both_owners_are_low(self) -> None:
        decision = closeout.decide_next_owner(
            _verifier_with_inputs(
                paint_widget_p95=80,
                canvas_p95=50,
                renderer_text_p95=70,
            )
        )

        self.assertEqual("decided", decision["status"])
        self.assertEqual("no-code-change", decision["owner"])
        self.assertEqual("lock-gates-and-docs", decision["action"])

    def test_owner_decision_is_incomplete_without_valid_artifacts(self) -> None:
        decision = closeout.decide_next_owner({"ok": False})

        self.assertEqual("incomplete", decision["status"])
        self.assertEqual("wait-for-valid-artifacts", decision["action"])

    def test_non_dry_run_stops_before_repo_gates_when_verifier_fails(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            report = root / "closeout-summary.json"
            verifier = {
                "ok": False,
                "validation": {"date_tag": "run-a", "errors": ["missing stats"]},
                "attribution": {"date_tag": "run-a-attrib", "errors": []},
            }
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_closeout.py",
                    str(root / "validation-dir"),
                    "--attribution-dir",
                    str(root / "attribution-dir"),
                    "--out-report",
                    str(report),
                ],
            ):
                with patch.object(closeout.verify, "verify_artifact_dirs", return_value=verifier):
                    with patch.object(closeout, "_run") as run_gate:
                        with redirect_stderr(io.StringIO()):
                            rc = closeout.main()

            summary = json.loads(report.read_text(encoding="utf-8"))

        self.assertEqual(1, rc)
        self.assertFalse(summary["ok"])
        self.assertEqual("run-a", summary["validation_date_tag"])
        self.assertEqual("run-a-attrib", summary["attribution_date_tag"])
        self.assertEqual([], summary["steps"])
        run_gate.assert_not_called()

    def test_non_dry_run_records_gate_results_after_verifier_passes(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            report = root / "closeout-summary.json"
            verifier = {
                "ok": True,
                "validation": {"date_tag": "run-a"},
                "attribution": {"date_tag": "run-a-attrib"},
            }
            gate_result = {
                "cmd": ["python", "--version"],
                "rc": 0,
                "elapsed_ms": 1,
                "stdout": str(root / "stdout.log"),
                "stderr": str(root / "stderr.log"),
            }
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_closeout.py",
                    str(root / "validation-dir"),
                    "--attribution-dir",
                    str(root / "attribution-dir"),
                    "--out-report",
                    str(report),
                ],
            ):
                with patch.object(closeout.verify, "verify_artifact_dirs", return_value=verifier):
                    with patch.object(closeout, "_run", return_value=gate_result) as run_gate:
                        with redirect_stdout(io.StringIO()):
                            rc = closeout.main()

            summary = json.loads(report.read_text(encoding="utf-8"))

        self.assertEqual(0, rc)
        self.assertTrue(summary["ok"])
        self.assertEqual("run-a", summary["validation_date_tag"])
        self.assertEqual("run-a-attrib", summary["attribution_date_tag"])
        self.assertEqual(4, len(summary["steps"]))
        self.assertEqual(4, run_gate.call_count)

    def test_non_windows_flag_is_passed_to_verifier(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            report = root / "closeout-summary.json"
            verifier = {
                "ok": True,
                "validation": {"date_tag": "run-a"},
                "attribution": {"date_tag": "run-a-attrib"},
            }
            gate_result = {
                "cmd": ["python", "--version"],
                "rc": 0,
                "elapsed_ms": 1,
                "stdout": str(root / "stdout.log"),
                "stderr": str(root / "stderr.log"),
            }
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_closeout.py",
                    str(root / "validation-dir"),
                    "--attribution-dir",
                    str(root / "attribution-dir"),
                    "--allow-non-windows",
                    "--out-report",
                    str(report),
                ],
            ):
                with patch.object(closeout.verify, "verify_artifact_dirs", return_value=verifier) as verify_dirs:
                    with patch.object(closeout, "_run", return_value=gate_result):
                        with redirect_stdout(io.StringIO()):
                            rc = closeout.main()
            summary = json.loads(report.read_text(encoding="utf-8"))

        self.assertEqual(0, rc)
        self.assertTrue(summary["allow_non_windows"])
        verify_dirs.assert_called_once_with(
            root / "validation-dir",
            root / "attribution-dir",
            allow_non_windows=True,
        )


if __name__ == "__main__":
    unittest.main()
