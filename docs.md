# pres

A terminal markdown slide presenter.

Built with Rust + Ratatui

Press `Space` to go to the next slide.

---

## Install

```bash
cargo install pres
```

Or build from source:

```bash
git clone https://github.com/dvjn/pres.git
cd pres
cargo install --locked --path .
```

---

## Quick Start

```bash
pres slides.md
```

Present your markdown file in the terminal.

---

## Navigation

- `Space` / `n` / `Enter` — Next slide
- `p` / `Backspace` — Previous slide

Horizontal (between sections):

- `→` / `l` — Next section
- `←` / `h` — Previous section

Vertical (within a section):

- `↓` / `j` — Down
- `↑` / `k` — Up

Other:

- `g` — First slide
- `G` — Last slide
- `q` / `Esc` — Quit

Look at the **bottom right** for your current slide position.

-----

### Navigation — Slide Structure

Slides are organized in a **grid**: sections go left/right, and each section can have vertical slides going up/down.

`---` in your markdown creates a new **section** (horizontal).

`-----` creates a new **vertical slide** within the current section.

-----

### Navigation — Markdown

```markdown
First section, first vertical slide

-----

First section, second vertical slide

---

Second section
```

---

## Headings

# H1 Heading

## H2 Heading

### H3 Heading

Each heading level has its own distinct style.

-----

### Headings — Markdown

```markdown
# H1 Heading

## H2 Heading

### H3 Heading
```

---

## Text Styles

**Bold text** draws attention to key points.

*Italic text* adds subtle emphasis.

**Bold with *nested italic* inside** — both modifiers apply.

~~Strikethrough~~ for crossed out text.

You can mix inline `code` with **bold** and *italic* in the same line.

-----

### Text Styles — Markdown

```markdown
**Bold text** draws attention to key points.

*Italic text* adds subtle emphasis.

**Bold with *nested italic* inside** — both modifiers apply.

~~Strikethrough~~ for crossed out text.

You can mix inline `code` with **bold** and *italic* in the same line.
```

---

## Blockquotes

Single-line:

> The best tool is the one you actually use.

Multi-line:

> Simple things should be simple,
> complex things should be possible.
> — Alan Kay

-----

### Blockquotes — Markdown

```markdown
> The best tool is the one you actually use.

> Simple things should be simple,
> complex things should be possible.
> — Alan Kay
```

---

## Lists

- Alpha
- Beta
- Gamma

Ordered:

1. Install Rust
2. Clone the repo
3. Run `cargo build`

-----

### Lists — Nested

- Frontend
  1. HTML
  2. CSS
  3. JavaScript
- Backend
  1. Rust
  2. Go

-----

### Lists — Deeper Nesting

- Tools
  - Build
    - `cargo build`
    - `cargo check`
  - Test
    - `cargo test`
    - `cargo bench`
- Deploy
  - Staging
    1. Build release binary
    2. Upload artifact
    3. Smoke test
  - Production
    1. Tag release
    2. Deploy

-----

### Lists — Markdown

```markdown
- Alpha
- Beta

1. First
2. Second

- Parent
  1. Nested ordered
  - Nested unordered
    - Deeper
```

---

## Task Lists

- [x] Implement strikethrough
- [x] Implement task lists
- [x] Implement links
- [x] Implement tables
- [ ] Add more features

-----

### Task Lists — Markdown

```markdown
- [x] Done item
- [ ] Pending item
```

---

## Code Blocks

```rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

-----

### Code Blocks — More Languages

```typescript
interface Slide {
  raw: string;
}

function render(slide: Slide): string[] {
  return slide.raw.split("\n");
}
```

```bash
pres slides.md
```

-----

### Code Blocks — Markdown

````markdown
```rust
fn main() {
    println!("Hello!");
}
```

Inline code: `render(slide, theme)`
````

---

## Tables

| Language | Paradigm   | Year |
| -------- | ---------- | ---- |
| Rust     | Systems    | 2010 |
| Python   | Scripting  | 1991 |
| Haskell  | Functional | 1990 |
| Go       | Concurrent | 2009 |

-----

### Tables — Alignment

| Left  | Center | Right |
| :---- | :----: | ----: |
| alpha |  beta  | gamma |
| 1     |   2    |     3 |

-----

### Tables — Markdown

```markdown
| Left  | Center | Right |
| :---- | :----: | ----: |
| alpha |  beta  | gamma |
| 1     |   2    |     3 |
```

---

## Links

Visit [the Rust website](https://www.rust-lang.org) for more info.

Check out [Ratatui](https://ratatui.rs) for terminal UI.

-----

### Links — Markdown

```markdown
Visit [the Rust website](https://www.rust-lang.org) for more info.
```

---

## Putting It All Together

Here is **bold**, *italic*, and `inline code` in one paragraph.

> Use the right tool for the job.

```rust
fn main() {
    println!("pres!");
}
```

Steps to run:

1. Write your slides in Markdown
2. Separate sections with `---`
3. Separate vertical slides with `-----`

---

# End

You have seen all supported features.

Press `g` to jump back to the first slide, or `q` to quit.
