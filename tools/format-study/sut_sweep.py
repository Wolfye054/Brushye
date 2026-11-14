#!/usr/bin/env python3
"""Differential sweep over .sut samples (study-plan.md method 1 and 3).

    sut_sweep.py BASE.sut SAMPLE.sut...   per sample: every table cell that differs from BASE

Variant rows are matched by role, as the Node row names them: "current" (NodeVariantID) and
"init" (NodeInitVariantID); other rows by _PW_ID. Fields that differ between two exports of
one sub tool or between sub tools (ids, uuids, counters; see sut.md) are ignored unless --all.
Blobs are shown as length + hex (shortened). Stdlib only; files opened read-only.
"""

import sqlite3
import sys
from pathlib import Path

VOLATILE = {
    "Manager": {"RootUuid", "MaxVariantID", "SavedCount", "CurrentNodeUuid", "ObjectNodeUuid"},
    "Node": {"NodeUuid", "NodeVariantID", "NodeInitVariantID"},
    "Variant": {"VariantID"},
}
SHOW_ALL = False


def load(path):
    con = sqlite3.connect(f"file:{path}?mode=ro&immutable=1", uri=True)
    out = {}
    for (t,) in con.execute("SELECT name FROM sqlite_master WHERE type='table'"):
        if t == "sqlite_sequence":
            continue
        cur = con.execute(f'SELECT * FROM "{t}"')
        cols = [d[0] for d in cur.description]
        out[t] = {rec["_PW_ID"]: rec for rec in (dict(zip(cols, r)) for r in cur)}
    con.close()
    node = next(iter(out.get("Node", {}).values()), {})
    roles = {node.get("NodeVariantID"): "current", node.get("NodeInitVariantID"): "init"}
    out["Variant"] = {
        roles.get(r["VariantID"], f"id{r['VariantID']}"): r for r in out.get("Variant", {}).values()
    }
    return out


def show(v, n=48):
    if isinstance(v, bytes):
        h = v.hex()
        return f"<{len(v)}B {h[:n]}{'…' if len(h) > n else ''}>"
    return repr(v)


def diff(a, b):
    out = []
    for t in sorted(set(a) | set(b)):
        ra, rb = a.get(t, {}), b.get(t, {})
        for k in sorted(set(ra) | set(rb), key=str):
            if k not in ra or k not in rb:
                out.append(f"{t}[{k}]: row {'added' if k in rb else 'removed'}")
                continue
            for c in ra[k]:
                if c == "_PW_ID" or (not SHOW_ALL and c in VOLATILE.get(t, ())):
                    continue
                if ra[k].get(c) != rb[k].get(c):
                    out.append(f"{t}[{k}].{c}: {show(ra[k].get(c))} -> {show(rb[k].get(c))}")
    return out


def main(argv):
    global SHOW_ALL
    SHOW_ALL = "--all" in argv
    argv = [a for a in argv if a != "--all"]
    if len(argv) < 2:
        sys.exit(__doc__)
    base = load(argv[0])
    for p in argv[1:]:
        print(f"== {Path(p).name}")
        for line in diff(base, load(p)) or ["(no differences)"]:
            print("  " + line)


if __name__ == "__main__":
    main(sys.argv[1:])
