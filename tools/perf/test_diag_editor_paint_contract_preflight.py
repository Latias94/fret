import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import diag_editor_paint_contract_preflight as preflight


class EditorPaintContractPreflightTests(unittest.TestCase):
    def test_required_contract_inputs_exist(self) -> None:
        root = preflight._workspace_root()
        required = [*preflight.EDITOR_PROBE_SCRIPTS, preflight.BASELINE_MATRIX]

        missing = [path for path in required if not (root / path).is_file()]

        self.assertEqual([], missing)

    def test_editor_probe_scripts_disable_torture_overlay_by_default(self) -> None:
        root = preflight._workspace_root()

        failures = []
        for script in preflight.EDITOR_PROBE_SCRIPTS:
            check = preflight.check_script_contract(root / script)
            if check["rc"] != 0:
                failures.append((script, check["stderr"]))

        self.assertEqual([], failures)

    def test_script_contract_rejects_missing_overlay_default(self) -> None:
        with TemporaryDirectory() as td:
            script = Path(td) / "script.json"
            script.write_text('{"schema_version": 2, "meta": {"env_defaults": {}}, "steps": []}', encoding="utf-8")

            check = preflight.check_script_contract(script)

        self.assertEqual(1, check["rc"])
        self.assertIn("FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY", str(check["stderr"]))


if __name__ == "__main__":
    unittest.main()
