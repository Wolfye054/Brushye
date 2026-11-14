# Bundled fonts

| File | Font | Source | License |
|---|---|---|---|
| `Go-Regular.ttf`, `Go-Medium.ttf`, `Go-Mono.ttf` | Go fonts (Bigelow & Holmes, for the Go project) | `golang.org/x/image/font/gofont/ttfs` (github.com/golang/image, master, fetched 2026-11-13) | BSD-3-Clause, `LICENSE-GoFonts.txt` |
| `Phosphor.ttf`, `Phosphor-Fill.ttf` | Phosphor Icons 2.1.2 (regular and fill weights) | npm `@phosphor-icons/web` 2.1.2 | MIT, `LICENSE-Phosphor.txt` |

Both licenses are on the project's allow list (`deny.toml`). `src/icons.rs` is generated from
Phosphor's `regular/style.css` by `tools/gen_phosphor_icons.py`.
