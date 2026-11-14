# Format study tools

Black-box tools for the .sut/.clip format study (`specs/formats/study-plan.md`). They run only
on sample files the maintainer owns (DECISIONS D-020). They recognise standard containers by
their published magic numbers (SQLite, ZIP, zlib, PNG) and assume nothing about CSP's own
layout. Python standard library only.

- `dump.py [--json] FILE`: a structural dump (container, tables or entries, rows, blobs with
  length, hash, leading bytes, and embedded zlib or PNG data).
- `diff.py [--bytes] A B`: every difference between two dumps, as paths (table / row /
  column). With `--bytes` it also diffs changed blobs byte by byte, decompressed when they are
  zlib.

Each finding in `specs/formats/sut.md` cites the sample pair and the diff path it came from.
