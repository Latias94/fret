import unittest

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


if __name__ == "__main__":
    unittest.main()
