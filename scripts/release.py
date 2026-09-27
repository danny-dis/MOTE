"""Small, stdlib-only release packager. No workspace or credentials are copied."""
import argparse
import hashlib
from pathlib import Path
import re
import shutil
import tempfile
import tomllib

TARGETS = ("x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin")


def check_tag(root, tag):
    version = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag) or tag != "v" + version:
        raise ValueError("release tag must be vMAJOR.MINOR.PATCH and match Cargo.toml")


def package(root, binary, output, target, tag):
    check_tag(root, tag)
    if target not in TARGETS:
        raise ValueError("unsupported release target")
    output.mkdir(parents=True, exist_ok=True)
    name = f"mote-{tag}-{target}"
    suffix = ".exe" if "windows" in target else ""
    with tempfile.TemporaryDirectory() as tmp:
        stage = Path(tmp) / name
        stage.mkdir()
        for executable in ("mote", "mote-bridge"):
            shutil.copy2(binary / (executable + suffix), stage / (executable + suffix))
            (stage / (executable + suffix)).chmod(0o755)
        for source, dest in (("LICENSE", "LICENSE"), ("THIRD_PARTY_NOTICES.md", "THIRD_PARTY_NOTICES.md"),
                             ("docs/QUICKSTART.md", "QUICKSTART.md"), ("examples/release-agent.yaml", "agent.yaml")):
            shutil.copy2(root / source, stage / dest)
        for notice in ("THIRD_PARTY_LICENSES.txt", "RUST_COPYRIGHT.html"):
            shutil.copy2(binary / notice, stage / notice)
        (stage / "workspace").mkdir()
        (stage / "workspace/.keep").touch()
        archive = Path(shutil.make_archive(str(output / name), "zip" if suffix else "gztar", tmp, name))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
    return archive


def assemble(root, output, tag):
    """Require every platform and validate upload digests before publication."""
    check_tag(root, tag)
    lines = []
    for target in TARGETS:
        extension = "zip" if "windows" in target else "tar.gz"
        archive = output / f"mote-{tag}-{target}.{extension}"
        checksum = archive.with_name(archive.name + ".sha256")
        if not archive.is_file() or not checksum.is_file():
            raise ValueError(f"missing release asset: {archive.name}")
        expected = hashlib.sha256(archive.read_bytes()).hexdigest() + "  " + archive.name + "\n"
        if checksum.read_text(encoding="utf-8") != expected:
            raise ValueError(f"checksum mismatch: {archive.name}")
        lines.append(expected)
    sums = output / "SHA256SUMS"
    sums.write_text("".join(sorted(lines)), encoding="utf-8")
    return sums


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--binary-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--target", choices=TARGETS, required=True)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args()
    print(package(args.root, args.binary_dir, args.output, args.target, args.tag))
