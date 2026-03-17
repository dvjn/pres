# pres

A terminal markdown slide presenter.

## Install

```bash
cargo install pres
```

## Usage

```bash
pres slides.md
pres --watch slides.md  # auto-reload on file changes
```

## Slides

Use `---` to separate sections (horizontal) and `-----` for vertical slides within a section.

## Navigation

| Key                     | Action             |
| ----------------------- | ------------------ |
| `Space` / `n` / `Enter` | Next slide         |
| `p` / `Backspace`       | Previous slide     |
| `→` / `l`               | Next section       |
| `←` / `h`               | Previous section   |
| `↓` / `j`               | Slide down         |
| `↑` / `k`               | Slide up           |
| `g` / `G`               | First / last slide |
| `q` / `Esc`             | Quit               |

For a full feature showcase, see [`docs.md`](docs.md).

## License

MIT
