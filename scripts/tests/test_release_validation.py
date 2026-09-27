import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("release", Path(__file__).resolve().parents[1] / "release.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ValidationTests(unittest.TestCase):
    def test_tag_must_match_manifest_and_be_stable_version(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.toml").write_text('[package]\nversion = "0.13.0"\n')
            release.check_tag(root, "v0.13.0")
            for tag in ("v0.13.1", "../v0.13.0", "v0.13.0-rc1", "0.13.0"):
                with self.subTest(tag=tag), self.assertRaises(ValueError):
                    release.check_tag(root, tag)

    def test_assemble_requires_all_targets_and_detects_tampering(self):
        self.assertTrue(hasattr(release, "assemble"), "checksum assembly is not implemented")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.toml").write_text('[package]\nversion = "0.13.0"\n')
            output = root / "assets"
            output.mkdir()
            with self.assertRaises(ValueError):
                release.assemble(root, output, "v0.13.0")
            expected = []
            for target in release.TARGETS:
                extension = "zip" if "windows" in target else "tar.gz"
                archive = output / f"mote-v0.13.0-{target}.{extension}"
                archive.write_bytes(b"archive fixture")
                line = hashlib.sha256(archive.read_bytes()).hexdigest() + "  " + archive.name + "\n"
                archive.with_name(archive.name + ".sha256").write_text(line)
                expected.append(line)
            result = release.assemble(root, output, "v0.13.0")
            self.assertEqual(result.read_text(), "".join(sorted(expected)))
            archive.write_bytes(b"tampered")
            with self.assertRaises(ValueError):
                release.assemble(root, output, "v0.13.0")


if __name__ == "__main__":
    unittest.main()
