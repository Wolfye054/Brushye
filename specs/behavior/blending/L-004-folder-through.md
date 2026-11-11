# L-004 Layer folders: Normal vs Through (pass-through)
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK

## Steps
1. base; folder F containing top (Multiply). Export with F = Normal and F = Through.
2. Same with F opacity 50 %.
3. Folder contains two layers (Screen over Multiply).

## What to measure
A Normal folder composites its contents in isolation (on transparent) and then blends the
result. Through blends its children straight into the backdrop. Verify both and the exact numbers.
