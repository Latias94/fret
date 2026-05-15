import json
import io
import sys
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

import diag_editor_paint_contract_closeout as closeout


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


if __name__ == "__main__":
    unittest.main()
