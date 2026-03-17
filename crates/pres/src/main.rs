#![warn(clippy::pedantic)]

use std::{io, panic, path::PathBuf, sync::mpsc};

use clap::Parser;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify::{RecursiveMode, Watcher, recommended_watcher};
use ratatui::{Terminal, backend::CrosstermBackend};

mod app;
mod parser;
mod renderer;
mod theme;
mod ui;

use theme::Theme;

#[derive(Parser)]
#[command(name = "pres", about = "present markdown slides in the terminal")]
struct Cli {
    /// Markdown file to present
    file: PathBuf,

    /// Reload slides automatically when the file changes
    #[arg(long, short)]
    watch: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let contents = std::fs::read_to_string(&cli.file).map_err(|e| {
        eprintln!("Error reading {}: {}", cli.file.display(), e);
        e
    })?;

    let sections = parser::parse(&contents);
    if sections.is_empty() {
        eprintln!("No slides found in {}", cli.file.display());
        return Ok(());
    }

    run_terminal(sections, cli.file, cli.watch)
}

fn run_terminal(sections: Vec<parser::Section>, file: PathBuf, watch: bool) -> io::Result<()> {
    let filename = file
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    // Set up file watcher before entering raw mode so errors surface cleanly.
    let reload_rx = if watch {
        let (tx, rx) = mpsc::channel::<()>();
        let file_path = file.canonicalize().unwrap_or_else(|_| file.clone());
        let watch_file = file_path.clone();
        let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                use notify::EventKind::{Create, Modify, Remove};
                let is_relevant = matches!(event.kind, Modify(_) | Create(_) | Remove(_))
                    && event.paths.iter().any(|p| p == &watch_file);
                if is_relevant {
                    let _ = tx.send(());
                }
            }
        })
        .map_err(|e| io::Error::other(format!("watcher error: {e}")))?;

        // Watch the parent directory so rename-based saves (vim, neovim, etc.) are caught.
        let watch_dir = file_path.parent().unwrap_or(&file_path);
        watcher
            .watch(watch_dir, RecursiveMode::NonRecursive)
            .map_err(|e| io::Error::other(format!("watch error: {e}")))?;

        Some((rx, file, watcher))
    } else {
        None
    };

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let result = run(
        sections,
        &filename,
        reload_rx.map(|(rx, path, _watcher)| (rx, path)),
    );

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

#[allow(clippy::needless_pass_by_value)]
fn run(
    sections: Vec<parser::Section>,
    filename: &str,
    reload_rx: Option<(mpsc::Receiver<()>, PathBuf)>,
) -> io::Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let theme = Theme::default();

    let mut current_sections = sections;
    let mut rendered = renderer::render_all(&current_sections, &theme);
    let mut presenter = app::App::new(&current_sections, filename.to_string());

    loop {
        terminal.draw(|f| ui::draw(f, &presenter, &rendered, &theme))?;

        // Check for file reload signal before blocking on keyboard input.
        if let Some((ref rx, ref path)) = reload_rx
            && rx.try_recv().is_ok()
        {
            while rx.try_recv().is_ok() {}

            if let Ok(contents) = std::fs::read_to_string(path) {
                let new_sections = parser::parse(&contents);
                if !new_sections.is_empty() {
                    current_sections = new_sections;
                    rendered = renderer::render_all(&current_sections, &theme);
                    presenter = app::App::new_at(
                        &current_sections,
                        filename.to_string(),
                        presenter.col,
                        presenter.row,
                    );
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char(' ' | 'n') | KeyCode::Enter => presenter.next(),
                KeyCode::Char('p') | KeyCode::Backspace => presenter.prev(),

                KeyCode::Right | KeyCode::Char('l') => presenter.right(),
                KeyCode::Left | KeyCode::Char('h') => presenter.left(),
                KeyCode::Down | KeyCode::Char('j') => presenter.down(),
                KeyCode::Up | KeyCode::Char('k') => presenter.up(),

                KeyCode::Char('g') => presenter.first(),
                KeyCode::Char('G') => presenter.last(),

                KeyCode::Char('q') | KeyCode::Esc => break,

                _ => {}
            }
        }
    }

    Ok(())
}
