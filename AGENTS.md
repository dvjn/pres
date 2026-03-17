# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## What is this?

`pres` is a terminal markdown slide presenter built with Rust, Ratatui, and crossterm. It renders markdown files as slides in the terminal with syntax highlighting, tables, lists, and text styling.

## Build & Development

```bash
cargo build                    # Build
cargo run -- slides.md         # Run with a markdown file
cargo test                     # Run all tests
cargo clippy                   # Lint
cargo fmt                      # Format
```

Requires Rust edition 2024.

## Repository Structure

This is a Cargo workspace with a single crate:

```
crates/
└── pres/
    ├── assets/
    │   └── github-dark.tmTheme   # Bundled syntax highlight theme
    └── src/
        ├── main.rs      # CLI entry point, terminal setup, keyboard event loop
        ├── app.rs       # Navigation state (App struct)
        ├── ui.rs        # Drawing: current slide + status bar into a ratatui frame
        ├── parser.rs    # Markdown → Section/Slide grid
        ├── renderer.rs  # Slide → Vec<Line<'static>> with styled spans, render_all()
        └── theme.rs     # Theme struct with styles for all visual elements
```

### Source modules

- **`parser.rs`** — Splits markdown input into a 2D grid of `Section`s (horizontal, separated by `---`) each containing `Slide`s (vertical, separated by `-----`). Uses pulldown-cmark with offset iteration to find split points by inspecting the raw source length of `Rule` events, then slices raw markdown accordingly.
- **`renderer.rs`** — Converts each `Slide`'s raw markdown into `Vec<Line<'static>>` (ratatui lines with styled spans). Handles headings, text styles, code blocks (with syntect highlighting), blockquotes, lists, task lists, tables, and links. Also exposes `render_all()` to pre-render all slides up front.
- **`theme.rs`** — Defines the `Theme` struct with styles for all visual elements. Colors are inspired by charmbracelet/glow. Bundles `assets/github-dark.tmTheme` via `include_bytes!`.
- **`app.rs`** — Pure navigation state (`App` struct with `col`/`row` into the section/slide grid and `slide_counts: Vec<usize>` for bounds). Navigation methods: `next`, `prev`, `left`, `right`, `up`, `down`, `first`, `last`. `App::new_at` constructs with a clamped starting position (used on file reload).
- **`ui.rs`** — `draw()` renders the current slide centred in the terminal area and delegates to `draw_statusline()` for the filename and position counter at the bottom.
- **`main.rs`** — CLI entry point (clap, `--watch` flag), terminal setup (crossterm raw mode, alternate screen, panic hook), file watcher (notify, watches parent directory for rename-based saves), and keyboard event loop.
