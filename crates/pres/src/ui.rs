use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::Paragraph,
};

use crate::{app::App, renderer::RenderedSections, theme::Theme};

pub fn draw(f: &mut ratatui::Frame, app: &App, rendered: &RenderedSections, theme: &Theme) {
    let area = f.area();

    if app.slide_counts.is_empty() {
        return;
    }

    let status_height = 1;

    let slide = &rendered.sections[app.col][app.row];
    let content_height = u16::try_from(slide.lines.len()).unwrap_or(u16::MAX);
    let max_width: u16 = 80.min(area.width);

    let available_height = area.height.saturating_sub(status_height);
    let top_pad = available_height.saturating_sub(content_height) / 2;

    let vertical = Layout::vertical([
        Constraint::Length(top_pad),
        Constraint::Length(content_height.min(available_height)),
        Constraint::Min(0),
    ]);
    let [_, content_area, _] = vertical.areas(area);

    let h_pad = area.width.saturating_sub(max_width) / 2;
    let horizontal = Layout::horizontal([
        Constraint::Length(h_pad),
        Constraint::Length(max_width),
        Constraint::Min(0),
    ]);
    let [_, slide_area, _] = horizontal.areas(content_area);

    f.render_widget(
        Paragraph::new(slide.lines.clone()).alignment(Alignment::Left),
        slide_area,
    );

    draw_statusline(f, app, theme, area);
}

fn draw_statusline(f: &mut ratatui::Frame, app: &App, theme: &Theme, area: Rect) {
    let bottom_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    f.render_widget(
        Paragraph::new(format!(" {} ", app.filename))
            .style(theme.counter)
            .alignment(Alignment::Left),
        bottom_area,
    );

    let depth = app.slide_counts[app.col];
    let position = if depth > 1 {
        format!(
            " ← {} / {} →  ↑ {} / {} ↓ ",
            app.col + 1,
            app.slide_counts.len(),
            app.row + 1,
            depth
        )
    } else {
        format!(" ← {} / {} → ", app.col + 1, app.slide_counts.len())
    };
    f.render_widget(
        Paragraph::new(position)
            .style(theme.counter)
            .alignment(Alignment::Right),
        bottom_area,
    );
}
