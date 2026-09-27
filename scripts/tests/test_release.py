"""Archive tests use fixture bytes, not claims of executable validation."""
import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest
import zipfile

SCRIPT = Path(__file__).resolve().parents[1] / "release.py"


class ReleaseTests(unittest.TestCase):
    def test_windows_archive_has_only_distributable_files_and_checksum(self):
        self.assertTrue(SCRIPT.is_file(), "release packager is not implemented")
        spec = importlib.util.spec_from_file_location("release", SCRIPT)
        release = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(release)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.toml").write_text('[package]\nversion = "0.13.0"\n')
            for name in ("LICENSE", "THIRD_PARTY_NOTICES.md", "docs/QUICKSTART.md", "examples/release-agent.yaml"):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("fixture")
            binary = root / "bin"
            binary.mkdir()
            for name in ("mote.exe", "mote-bridge.exe"):
                (binary / name).write_bytes(b"fixture-binary")
            (binary / "secret.env").write_text("not distributable")
            for name in ("THIRD_PARTY_LICENSES.txt", "RUST_COPYRIGHT.html"):
                (binary / name).write_text("license fixture")
            out = root / "out"
            archive = release.package(root, binary, out, "x86_64-pc-windows-msvc", "v0.13.0")
            with zipfile.ZipFile(archive) as zipped:
                members = {item.filename for item in zipped.infolist() if not item.is_dir()}
            prefix = "mote-v0.13.0-x86_64-pc-windows-msvc/"
            self.assertEqual(members, {prefix + name for name in (
                "mote.exe", "mote-bridge.exe", "LICENSE", "THIRD_PARTY_NOTICES.md",
                "QUICKSTART.md", "agent.yaml", "workspace/.keep",
                "THIRD_PARTY_LICENSES.txt", "RUST_COPYRIGHT.html")})
            self.assertEqual(archive.with_name(archive.name + ".sha256").read_text(),
                             hashlib.sha256(archive.read_bytes()).hexdigest() + "  " + archive.name + "\n")


if __name__ == "__main__":
    unittest.main()
