#!/usr/bin/env python3
"""Differential comparison of two sample files (study-plan.md method 1).

    diff.py A B          print every difference between the two structural dumps
    diff.py --bytes A B  also diff changed blobs byte by byte (their zlib-decompressed form
                         when both decompress)

Rows of SQLite tables are matched by position. Output lines are paths into the dump
(table / row / column) with the old and new value, so a finding can cite them directly.
"""

import sqlite3
import sys
import zlib

from dump import dump


def walk(a, b, path, out):
    if type(a) is not type(b):
        out.append((path, a, b))
    elif isinstance(a, dict):
        for k in sorted(set(a) | set(b), key=str):
            if k in ("file", "sha256") and path == "":
                continue
            if k not in a or k not in b:
                out.append((f"{path}/{k}", a.get(k, "<absent>"), b.get(k, "<absent>")))
            else:
                walk(a[k], b[k], f"{path}/{k}", out)
    elif isinstance(a, list):
        for i in range(max(len(a), len(b))):
            if i >= len(a) or i >= len(b):
                out.append((f"{path}[{i}]", a[i] if i < len(a) else "<absent>",
                            b[i] if i < len(b) else "<absent>"))
            else:
                walk(a[i], b[i], f"{path}[{i}]", out)
    elif a != b:
        out.append((path, a, b))


def sqlite_blobs(path):
    con = sqlite3.connect(f"file:{path}?mode=ro&immutable=1", uri=True)
    out = {}
    for (name,) in con.execute("SELECT name FROM sqlite_master WHERE type='table'"):
        q = '"' + name.replace('"', '""') + '"'
        cols = [r[1] for r in con.execute(f"PRAGMA table_info({q})")]
        for ri, row in enumerate(con.execute(f"SELECT * FROM {q}")):
            for ci, v in enumerate(row):
                if isinstance(v, bytes):
                    out[(name, ri, cols[ci])] = v
    con.close()
    return out


def byte_diff(x, y, limit=64):
    try:
        x, y, note = zlib.decompress(x), zlib.decompress(y), " (zlib-decompressed)"
    except zlib.error:
        note = ""
    lines = [f"    lengths {len(x)} -> {len(y)}{note}"]
    n = 0
    for i in range(max(len(x), len(y))):
        cx = x[i] if i < len(x) else None
        cy = y[i] if i < len(y) else None
        if cx != cy:
            lines.append(f"    @{i:#06x}: {cx!s:>4} -> {cy!s:<4}")
            n += 1
            if n >= limit:
                lines.append("    …")
                break
    return lines


def main(argv):
    with_bytes = "--bytes" in argv
    files = [a for a in argv if a != "--bytes"]
    if len(files) != 2:
        sys.exit(__doc__)
    a, b = dump(files[0]), dump(files[1])
    out = []
    walk(a, b, "", out)
    for path, x, y in out:
        print(f"{path}: {x!r} -> {y!r}")
    if not out:
        print("no structural differences")
    if with_bytes and a["container"]["kind"] == b["container"]["kind"] == "sqlite":
        ba, bb = sqlite_blobs(files[0]), sqlite_blobs(files[1])
        for key in sorted(set(ba) & set(bb), key=str):
            if ba[key] != bb[key]:
                print(f"blob {key}:")
                print("\n".join(byte_diff(ba[key], bb[key])))


if __name__ == "__main__":
    main(sys.argv[1:])
