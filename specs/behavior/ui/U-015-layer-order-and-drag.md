# U-015 Layer order with folders: drag and drop, Layer > Layer Order
- Status: **observed**
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-14 / automated-Wine (Xvfb :77, XTEST) / Claude
- Screenshots: `obs-out/m10/` on the observation machine (not committed).

## Setup (exact)
- Document: base.png, with `top_alpha Copy` and `top Copy` pasted above it. Ctrl+G on top
  Copy gives Folder 1 [top Copy] above top_alpha Copy and base.
- Default workspace. The Layer palette rows are 40 px high for layers and 30 px for folders.
  In the run the folder row spanned y 645–675 and the next row 676–715.
- **Drag:** an XTEST press on the `base` row, moves to a target y in the same column
  (x 1850), a release, then a palette screenshot, then Edit > Undo.
- **Layer Order:** Layer > Layer Order ▸ Top / Up / Down / Bottom (submenu at x ≈ 520).

## Observations
### Drag and drop (where the dragged layer lands)
While dragging, a ghost of the row follows the pointer. The landing place depends on the
pointer's height in the row it is released over:

| Row released on | Zone | Result |
|---|---|---|
| Layer | upper half | directly **above** that layer, in its folder |
| Layer | lower half | directly **below** that layer, in its folder |
| Folder | top quarter | directly **above** the folder (its sibling) |
| Folder | middle half | **into** the folder, **at the bottom** of its contents (also when collapsed; it stays collapsed) |
| Folder, expanded | bottom quarter | into the folder, **at the top** of its contents |
| Folder, collapsed | bottom quarter | directly **below** the folder (its sibling) |

- Boundaries measured on the 30 px folder row: y ≤ 651 above, 653–666 into, ≥ 668 top of
  contents. On a 40 px layer row: 694 above, 697 below.
- After a drop into a collapsed folder, the folder row is shown selected.
- The moved layer keeps its clip setting. Clip bars update: red when a base appears below it,
  grey when there is none.

### Layer > Layer Order
- **Up:**
  - If the sibling directly above is an **expanded folder**, the layer moves **into it, at the
    bottom**.
  - Otherwise it moves above that sibling. A **collapsed** folder is jumped over as one item.
  - At the top of a folder's contents, it **leaves the folder** to directly above it.
- **Down:** the mirror image.
  - An expanded folder below is entered at the **top**; a collapsed one is jumped over.
  - At the bottom of a folder's contents, the layer leaves the folder to directly below it.
- **Top / Bottom:** the top or bottom of the **whole stack, at the top level**, even from
  inside a folder.
- Sequences observed:
  - Up ×3 from just below Folder 1 [top Copy]: into the folder (below top Copy), then above
    top Copy, then above the folder.
  - Down ×3 from the top: the reverse.

## Open questions
- Drag and drop between documents, and dragging a folder into a folder by the middle zone:
  expected to be the same rules (a folder can't go into itself).
- The exact drop indicator drawing (hidden under the ghost row in the screenshots).
