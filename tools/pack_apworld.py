"""Zip world/nightreign into nightreign.apworld for Archipelago/custom_worlds/."""

from __future__ import annotations

import pathlib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = ROOT / "world" / "nightreign"
OUT = ROOT / "nightreign.apworld"


def main() -> None:
    if not SRC.exists():
        raise SystemExit(f"missing {SRC}")
    if OUT.exists():
        OUT.unlink()
    with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as zf:
        for path in SRC.rglob("*"):
            if path.is_dir():
                continue
            if path.suffix in {".pyc"} or path.name == "__pycache__":
                continue
            arc = pathlib.Path("nightreign") / path.relative_to(SRC)
            zf.write(path, arc.as_posix())
    print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
