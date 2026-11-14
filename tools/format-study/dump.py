#!/usr/bin/env python3
"""Structural dump of a sample file for the format study (specs/formats/study-plan.md).

Black-box only: it recognises standard containers by their published magic numbers (SQLite,
ZIP, zlib, PNG) and lists what is inside. It assumes nothing about CSP's own data layout.

    dump.py FILE            human-readable summary
    dump.py --json FILE     JSON (for diff.py)

Python standard library only. Files are opened read-only; SQLite in immutable mode.
"""

import hashlib
import json
import sqlite3
import sys
import zipfile
import zlib

PNG_SIG = b"\x89PNG\r\n\x1a\n"


def sha(b):
    return hashlib.sha256(b).hexdigest()


def blob_info(b):
    """Summary of a byte string: length, hash, leading bytes, and embedded standard data."""
    info = {"len": len(b), "sha256": sha(b)[:16], "head": b[:16].hex()}
    if b.startswith(PNG_SIG):
        info["png"] = png_info(b)
    try:
        d = zlib.decompress(b)
        info["zlib"] = {"len": len(d), "sha256": sha(d)[:16], "head": d[:16].hex()}
    except zlib.error:
        pass
    pngs = [i for i in find_all(b, PNG_SIG) if i > 0]
    if pngs:
        info["png_at"] = pngs[:20]
    return info


def find_all(b, needle):
    out, i = [], b.find(needle)
    while i != -1:
        out.append(i)
        i = b.find(needle, i + 1)
    return out


def png_info(b):
    # IHDR is the first chunk (PNG spec): width, height, bit depth, colour type.
    if len(b) < 33 or b[12:16] != b"IHDR":
        return {"valid": False}
    w = int.from_bytes(b[16:20], "big")
    h = int.from_bytes(b[20:24], "big")
    return {"w": w, "h": h, "depth": b[24], "colour_type": b[25]}


def cell(v):
    if isinstance(v, bytes):
        return {"blob": blob_info(v)}
    return v


def dump_sqlite(path):
    con = sqlite3.connect(f"file:{path}?mode=ro&immutable=1", uri=True)
    out = {"kind": "sqlite", "tables": {}}
    objs = con.execute(
        "SELECT type, name, sql FROM sqlite_master ORDER BY type, name"
    ).fetchall()
    out["schema"] = [{"type": t, "name": n, "sql": s} for t, n, s in objs]
    for t, name, _ in objs:
        if t != "table":
            continue
        q = '"' + name.replace('"', '""') + '"'
        cols = [r[1] for r in con.execute(f"PRAGMA table_info({q})")]
        rows = con.execute(f"SELECT * FROM {q}").fetchall()
        out["tables"][name] = {
            "columns": cols,
            "rows": [[cell(v) for v in r] for r in rows],
        }
    con.close()
    return out


def dump_zip(path):
    out = {"kind": "zip", "entries": []}
    with zipfile.ZipFile(path) as z:
        for e in z.infolist():
            data = z.read(e)
            out["entries"].append(
                {"name": e.filename, "method": e.compress_type, "data": blob_info(data)}
            )
    return out


def dump(path):
    with open(path, "rb") as f:
        raw = f.read()
    out = {"file": path, "len": len(raw), "sha256": sha(raw), "head": raw[:32].hex()}
    if raw.startswith(b"SQLite format 3\x00"):
        out["container"] = dump_sqlite(path)
    elif raw.startswith(b"PK\x03\x04"):
        out["container"] = dump_zip(path)
    else:
        out["container"] = {"kind": "unknown", "data": blob_info(raw)}
        sq = raw.find(b"SQLite format 3\x00")
        if sq > 0:
            out["container"]["sqlite_at"] = sq
    return out


def show(d, indent=0):
    pad = "  " * indent
    if isinstance(d, dict):
        for k, v in d.items():
            if isinstance(v, (dict, list)) and v:
                print(f"{pad}{k}:")
                show(v, indent + 1)
            else:
                print(f"{pad}{k}: {v}")
    elif isinstance(d, list):
        for i, v in enumerate(d):
            if isinstance(v, (dict, list)):
                print(f"{pad}[{i}]")
                show(v, indent + 1)
            else:
                print(f"{pad}- {v}")


def main(argv):
    as_json = "--json" in argv
    files = [a for a in argv if a != "--json"]
    if len(files) != 1:
        sys.exit(__doc__)
    d = dump(files[0])
    if as_json:
        json.dump(d, sys.stdout, indent=1)
        print()
    else:
        show(d)


if __name__ == "__main__":
    main(sys.argv[1:])
