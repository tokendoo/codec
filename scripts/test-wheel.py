#!/usr/bin/env python3
"""Test a built wheel in an isolated venv on Unix and Windows."""

import argparse
import os
from pathlib import Path
import subprocess
import tempfile
import venv
import zipfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wheel-dir", type=Path, default=Path("target/wheels"))
    args = parser.parse_args()
    wheels = sorted(args.wheel_dir.glob("tokendoo-*.whl"))
    if len(wheels) != 1:
        raise SystemExit("Expected exactly one Tokendoo wheel in the selected directory")
    wheel = wheels[0].resolve()
    with zipfile.ZipFile(wheel) as archive:
        names = archive.namelist()
        if not any(name == "LICENSE" or name.endswith("/LICENSE") for name in names):
            raise SystemExit("Wheel is missing LICENSE")
        if not any(name == "NOTICE" or name.endswith("/NOTICE") for name in names):
            raise SystemExit("Wheel is missing NOTICE")

    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="tokendoo-wheel-test-") as temporary:
        environment = Path(temporary) / "venv"
        venv.EnvBuilder(with_pip=True).create(environment)
        python = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        subprocess.run(
            [str(python), "-m", "pip", "install", "--no-index", "--no-deps", str(wheel)],
            cwd=temporary,
            check=True,
        )
        subprocess.run(
            [str(python), str(root / "crates/tokendoo-py/tests/test_package.py")],
            cwd=temporary,
            check=True,
        )
    print(f"Verified {wheel.name}: isolated install, package tests, LICENSE and NOTICE")


if __name__ == "__main__":
    main()
