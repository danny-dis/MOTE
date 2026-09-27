import importlib.util
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "licenses.py"


class LicenseTests(unittest.TestCase):
    def test_collects_actual_license_and_notice_text_and_rejects_missing(self):
        self.assertTrue(SCRIPT.is_file(), "license text collector is not implemented")
        spec = importlib.util.spec_from_file_location("licenses", SCRIPT)
        licenses = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(licenses)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "LICENSE-MIT").write_text("Copyright test owner\nPermission test text")
            (root / "NOTICE").write_text("Third-party attribution test")
            pkg = {"name": "test-crate", "version": "1.0.0", "source": "registry+test", "license": "MIT", "manifest_path": str(root / "Cargo.toml")}
            result = licenses.collect([pkg])
            self.assertIn("Copyright test owner", result)
            self.assertIn("Third-party attribution test", result)
            (root / "LICENSE-MIT").unlink()
            (root / "NOTICE").unlink()
            with self.assertRaises(ValueError):
                licenses.collect([pkg])


if __name__ == "__main__":
    unittest.main()
