use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use syntect::easy::HighlightLines;

use crate::{
    parser::{Section, Slide},
    theme::Theme,
};

#[derive(Clone, Default)]
pub struct RenderedSlide {
    pub lines: Vec<Line<'static>>,
}

#[derive(Clone)]
pub struct RenderedSections {
    pub sections: Vec<Vec<RenderedSlide>>,
}

type ListMarker = Option<u64>;

struct StyledText {
    text: String,
    style: Style,
}

type TableCell = Vec<StyledText>;
type TableRow = Vec<TableCell>;

#[allow(clippy::struct_excessive_bools)]
struct TableState {
    alignments: Vec<Alignment>,
    rows: Vec<TableRow>,
    in_head: bool,
    current_row: TableRow,
    current_cell: TableCell,
    in_bold: bool,
    in_italic: bool,
    in_strikethrough: bool,
}

pub fn render_all(sections: &[Section], theme: &Theme) -> RenderedSections {
    RenderedSections {
        sections: sections
            .iter()
            .map(|section| {
                section
                    .slides
                    .iter()
                    .map(|slide| render(slide, theme))
                    .collect()
            })
            .collect(),
    }
}

#[allow(clippy::too_many_lines)]
pub fn render(slide: &Slide, theme: &Theme) -> RenderedSlide {
    let ss = &theme.syntax_set;
    let syn_theme = &theme.syntax_highlight_theme;
    let mut result = RenderedSlide::default();
    let mut current_spans: Vec<Span<'static>> = Vec::new();

    let mut heading_level: Option<HeadingLevel> = None;
    let mut in_code_block = false;
    let mut in_blockquote = false;
    let mut in_bold = false;
    let mut in_italic = false;
    let mut in_strikethrough = false;
    let mut list_stack: Vec<ListMarker> = Vec::new();
    let mut list_counters: Vec<u64> = Vec::new();
    let mut code_block_lines: Vec<String> = Vec::new();
    let mut code_block_lang: String = String::new();
    let mut link_url: Option<String> = None;
    let mut link_text: Vec<String> = Vec::new();
    let mut table: Option<TableState> = None;

    let options = Options::all();
    let parser = Parser::new_ext(&slide.raw, options);

    let flush_line = |spans: &mut Vec<Span<'static>>, lines: &mut Vec<Line<'static>>| {
        let line = Line::from(std::mem::take(spans));
        lines.push(line);
    };

    for event in parser {
        if let Some(ref mut ts) = table {
            match &event {
                Event::Text(text) => {
                    let style = match (ts.in_bold, ts.in_italic, ts.in_strikethrough) {
                        (_, _, true) => Style::default().add_modifier(Modifier::CROSSED_OUT),
                        (true, true, _) => Style::default()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::ITALIC),
                        (true, false, _) => Style::default().add_modifier(Modifier::BOLD),
                        (false, true, _) => Style::default().add_modifier(Modifier::ITALIC),
                        (false, false, _) => Style::default(),
                    };
                    ts.current_cell.push(StyledText {
                        text: text.to_string(),
                        style,
                    });
                    continue;
                }
                Event::Code(text) => {
                    ts.current_cell.push(StyledText {
                        text: text.to_string(),
                        style: theme.code_inline,
                    });
                    continue;
                }
                Event::Start(Tag::Strong) => {
                    ts.in_bold = true;
                    continue;
                }
                Event::End(TagEnd::Strong) => {
                    ts.in_bold = false;
                    continue;
                }
                Event::Start(Tag::Emphasis) => {
                    ts.in_italic = true;
                    continue;
                }
                Event::End(TagEnd::Emphasis) => {
                    ts.in_italic = false;
                    continue;
                }
                Event::Start(Tag::Strikethrough) => {
                    ts.in_strikethrough = true;
                    continue;
                }
                Event::End(TagEnd::Strikethrough) => {
                    ts.in_strikethrough = false;
                    continue;
                }
                Event::Start(Tag::TableHead) => {
                    ts.in_head = true;
                    continue;
                }
                Event::End(TagEnd::TableHead) => {
                    ts.in_head = false;
                    let row = std::mem::take(&mut ts.current_row);
                    if !row.is_empty() {
                        ts.rows.insert(0, row);
                    }
                    continue;
                }
                Event::Start(Tag::TableRow) => {
                    ts.current_row = Vec::new();
                    continue;
                }
                Event::End(TagEnd::TableRow) => {
                    let row = std::mem::take(&mut ts.current_row);
                    ts.rows.push(row);
                    continue;
                }
                Event::Start(Tag::TableCell) => {
                    ts.current_cell = Vec::new();
                    continue;
                }
                Event::End(TagEnd::TableCell) => {
                    let cell = std::mem::take(&mut ts.current_cell);
                    ts.current_row.push(cell);
                    continue;
                }
                _ => {}
            }
        }

        match event {
            Event::Start(Tag::Table(alignments)) => {
                table = Some(TableState {
                    alignments: alignments.clone(),
                    rows: Vec::new(),
                    in_head: false,
                    current_row: Vec::new(),
                    current_cell: Vec::new(),
                    in_bold: false,
                    in_italic: false,
                    in_strikethrough: false,
                });
            }
            Event::End(TagEnd::Table) => {
                if let Some(ts) = table.take() {
                    result.lines.push(Line::default());
                    render_table(&ts, theme, &mut result.lines);
                    result.lines.push(Line::default());
                }
            }

            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
                result.lines.push(Line::default());
            }
            Event::End(TagEnd::Heading(_)) => {
                let (prefix_str, style, prefix_style) = match heading_level {
                    Some(HeadingLevel::H1) => ("# ", theme.h1, theme.h1_prefix),
                    Some(HeadingLevel::H2) => ("## ", theme.h2, theme.h2_prefix),
                    _ => ("### ", theme.h3, theme.h3_prefix),
                };
                let text: String = current_spans
                    .drain(..)
                    .map(|s| s.content.to_string())
                    .collect();
                result.lines.push(Line::from(vec![
                    Span::styled(prefix_str, prefix_style),
                    Span::styled(text, style),
                ]));
                result.lines.push(Line::default());
                heading_level = None;
            }

            Event::End(TagEnd::Paragraph) => {
                if !current_spans.is_empty() {
                    flush_line(&mut current_spans, &mut result.lines);
                }
                if !in_blockquote {
                    result.lines.push(Line::default());
                }
            }

            Event::Start(Tag::Strong) => in_bold = true,
            Event::End(TagEnd::Strong) => in_bold = false,

            Event::Start(Tag::Emphasis) => in_italic = true,
            Event::End(TagEnd::Emphasis) => in_italic = false,

            Event::Start(Tag::Strikethrough) => in_strikethrough = true,
            Event::End(TagEnd::Strikethrough) => in_strikethrough = false,

            Event::Start(Tag::Link { dest_url, .. }) => {
                link_url = Some(dest_url.to_string());
                link_text.clear();
            }
            Event::End(TagEnd::Link) => {
                if let Some(url) = link_url.take() {
                    let text = link_text.drain(..).collect::<String>();
                    current_spans.push(Span::styled(text, theme.link));
                    current_spans.push(Span::styled(
                        format!(" ({url})"),
                        theme.link.add_modifier(Modifier::DIM),
                    ));
                }
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_block_lines.clear();
                code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let inner_width = code_block_lines.iter().map(String::len).max().unwrap_or(0);
                let padded_width = inner_width + 4;
                let empty_line = " ".repeat(padded_width);

                let syntax = ss
                    .find_syntax_by_token(&code_block_lang)
                    .or_else(|| ss.find_syntax_by_name(&code_block_lang))
                    .unwrap_or_else(|| ss.find_syntax_plain_text());
                let theme_bg =
                    syn_theme
                        .settings
                        .background
                        .unwrap_or(syntect::highlighting::Color {
                            r: 0x27,
                            g: 0x28,
                            b: 0x22,
                            a: 0xff,
                        });
                let bg = Color::Rgb(theme_bg.r, theme_bg.g, theme_bg.b);
                let mut highlighter = HighlightLines::new(syntax, syn_theme);

                result.lines.push(Line::default());
                result.lines.push(Line::from(Span::styled(
                    empty_line.clone(),
                    Style::default().bg(bg),
                )));
                for code_line in code_block_lines.drain(..) {
                    let padding = " ".repeat(padded_width - code_line.len() - 2);
                    let padded_line = format!("  {code_line}{padding}");
                    let ranges = highlighter
                        .highlight_line(&padded_line, ss)
                        .unwrap_or_default();
                    let spans: Vec<Span<'static>> = ranges
                        .into_iter()
                        .map(|(style, text)| {
                            let fg = style.foreground;
                            Span::styled(
                                text.to_string(),
                                Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b)).bg(bg),
                            )
                        })
                        .collect();
                    result.lines.push(Line::from(spans));
                }
                result.lines.push(Line::from(Span::styled(
                    empty_line,
                    Style::default().bg(bg),
                )));
                result.lines.push(Line::default());
            }

            Event::Start(Tag::BlockQuote(_)) => {
                in_blockquote = true;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                in_blockquote = false;
                result.lines.push(Line::default());
            }

            Event::Start(Tag::List(start)) => {
                if !current_spans.is_empty() {
                    flush_line(&mut current_spans, &mut result.lines);
                }
                list_stack.push(start);
                list_counters.push(start.unwrap_or(1));
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
                list_counters.pop();
                if list_stack.is_empty() {
                    result.lines.push(Line::default());
                }
            }

            Event::Start(Tag::Item) => {
                let depth = list_stack.len();
                let indent = "  ".repeat(depth.saturating_sub(1));
                let prefix = match list_stack.last() {
                    Some(None) => format!("{indent}• "),
                    Some(Some(_)) => {
                        let counter = list_counters.last_mut().unwrap();
                        let s = format!("{indent}{counter}. ");
                        *counter += 1;
                        s
                    }
                    None => String::new(),
                };
                current_spans.push(Span::raw(prefix));
            }
            Event::End(TagEnd::Item) => {
                if !current_spans.is_empty() {
                    flush_line(&mut current_spans, &mut result.lines);
                }
            }

            Event::TaskListMarker(checked) => {
                // Replace the bullet span pushed by Start(Tag::Item)
                current_spans.pop();
                let depth = list_stack.len();
                let indent = "  ".repeat(depth.saturating_sub(1));
                if checked {
                    let marker = format!("{indent}[x] ");
                    current_spans.push(Span::styled(
                        marker,
                        Style::default().fg(Color::from_u32(0x0004_B575)),
                    ));
                } else {
                    let marker = format!("{indent}[ ] ");
                    current_spans.push(Span::styled(
                        marker,
                        Style::default().add_modifier(Modifier::DIM),
                    ));
                }
            }

            Event::Code(text) => {
                current_spans.push(Span::styled(text.to_string(), theme.code_inline));
            }

            Event::Text(text) => {
                if link_url.is_some() {
                    link_text.push(text.to_string());
                } else if in_code_block {
                    for line_text in text.lines() {
                        code_block_lines.push(line_text.to_string());
                    }
                } else if in_blockquote {
                    let style = theme.blockquote;
                    let prefix = Span::styled("│ ", style);
                    let content = Span::styled(text.to_string(), style);
                    if !current_spans.is_empty() {
                        flush_line(&mut current_spans, &mut result.lines);
                    }
                    result.lines.push(Line::from(vec![prefix, content]));
                } else {
                    let style = match (in_bold, in_italic, in_strikethrough) {
                        (_, _, true) => theme.strikethrough,
                        (true, true, _) => Style::default()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::ITALIC),
                        (true, false, _) => theme.bold,
                        (false, true, _) => theme.italic,
                        (false, false, _) => {
                            if heading_level.is_some() {
                                Style::default()
                            } else {
                                theme.normal
                            }
                        }
                    };
                    current_spans.push(Span::styled(text.to_string(), style));
                }
            }

            Event::SoftBreak | Event::HardBreak => {
                if !in_code_block && !current_spans.is_empty() {
                    flush_line(&mut current_spans, &mut result.lines);
                }
            }

            Event::Rule => {
                result.lines.push(Line::from(Span::styled(
                    "─".repeat(78),
                    Style::default().fg(Color::DarkGray),
                )));
                result.lines.push(Line::default());
            }

            _ => {}
        }
    }

    if !current_spans.is_empty() {
        result
            .lines
            .push(Line::from(std::mem::take(&mut current_spans)));
    }

    result
}

fn cell_text_len(cell: &[StyledText]) -> usize {
    cell.iter().map(|s| s.text.len()).sum()
}

fn render_table(ts: &TableState, theme: &Theme, lines: &mut Vec<Line<'static>>) {
    if ts.rows.is_empty() {
        return;
    }

    let col_count = ts.rows.iter().map(Vec::len).max().unwrap_or(0);
    if col_count == 0 {
        return;
    }

    // Compute max column widths based on text content
    let mut col_widths: Vec<usize> = vec![1; col_count];
    for row in &ts.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                col_widths[i] = col_widths[i].max(cell_text_len(cell));
            }
        }
    }

    let border_style = theme.table_border;
    let header_style = theme.table_header;

    // Build border rows
    let top = build_border(&col_widths, "┌", "┬", "┐", "─");
    let mid = build_border(&col_widths, "├", "┼", "┤", "─");
    let bot = build_border(&col_widths, "└", "┴", "┘", "─");

    lines.push(Line::from(Span::styled(top, border_style)));

    for (row_idx, row) in ts.rows.iter().enumerate() {
        let is_header = row_idx == 0;

        let mut spans = vec![Span::styled("│", border_style)];
        for (col_idx, width) in col_widths.iter().enumerate().take(col_count) {
            let align = ts
                .alignments
                .get(col_idx)
                .copied()
                .unwrap_or(Alignment::None);

            if let Some(cell) = row.get(col_idx) {
                let text_len = cell_text_len(cell);
                let pad_total = width.saturating_sub(text_len);

                let (left_pad, right_pad) = match align {
                    Alignment::Right => (pad_total, 0),
                    Alignment::Center => {
                        let lp = pad_total / 2;
                        (lp, pad_total - lp)
                    }
                    _ => (0, pad_total),
                };

                spans.push(Span::styled(
                    format!(" {}", " ".repeat(left_pad)),
                    Style::default(),
                ));
                for st in cell {
                    let final_style = if is_header {
                        st.style.patch(header_style)
                    } else {
                        st.style
                    };
                    spans.push(Span::styled(st.text.clone(), final_style));
                }
                spans.push(Span::styled(
                    format!("{} ", " ".repeat(right_pad)),
                    Style::default(),
                ));
            } else {
                let padded = pad_cell("", *width, align);
                spans.push(Span::styled(format!(" {padded} "), Style::default()));
            }
            spans.push(Span::styled("│", border_style));
        }
        lines.push(Line::from(spans));

        if is_header {
            lines.push(Line::from(Span::styled(mid.clone(), border_style)));
        }
    }

    lines.push(Line::from(Span::styled(bot, border_style)));
}

fn build_border(col_widths: &[usize], left: &str, sep: &str, right: &str, fill: &str) -> String {
    let mut s = left.to_string();
    for (i, &w) in col_widths.iter().enumerate() {
        s.push_str(&fill.repeat(w + 2));
        if i + 1 < col_widths.len() {
            s.push_str(sep);
        }
    }
    s.push_str(right);
    s
}

fn pad_cell(text: &str, width: usize, align: Alignment) -> String {
    let len = text.len();
    if len >= width {
        return text.to_string();
    }
    let pad = width - len;
    match align {
        Alignment::Right => format!("{text:>width$}"),
        Alignment::Center => {
            let left_pad = pad / 2;
            let right_pad = pad - left_pad;
            format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
        }
        _ => format!("{text:<width$}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Slide;
    use crate::theme::Theme;

    fn rendered_text(raw: &str) -> String {
        let theme = Theme::default();
        let slide = Slide {
            raw: raw.to_string(),
        };
        render(&slide, &theme)
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.to_string()))
            .collect::<String>()
    }

    #[test]
    fn headings_render_with_hash_prefix() {
        let out = rendered_text("# Title");
        assert!(out.contains("# "), "h1 should have '# ' prefix");

        let out = rendered_text("## Sub");
        assert!(out.contains("## "), "h2 should have '## ' prefix");

        let out = rendered_text("### Deep");
        assert!(out.contains("### "), "h3 should have '### ' prefix");
    }

    #[test]
    fn unordered_list_renders_bullet() {
        let out = rendered_text("- item");
        assert!(out.contains('•'), "unordered list item should use bullet •");
    }

    #[test]
    fn ordered_list_renders_number() {
        let out = rendered_text("1. first\n2. second");
        assert!(out.contains("1. "), "ordered list should show '1. '");
        assert!(out.contains("2. "), "ordered list should show '2. '");
    }

    #[test]
    fn link_renders_url_in_parentheses() {
        let out = rendered_text("[rust](https://rust-lang.org)");
        assert!(out.contains("rust"), "link text should appear");
        assert!(
            out.contains("(https://rust-lang.org)"),
            "link URL should appear in parens"
        );
    }

    #[test]
    fn blockquote_renders_bar_prefix() {
        let out = rendered_text("> quoted");
        assert!(out.contains("│ "), "blockquote should have '│ ' prefix");
    }

    #[test]
    fn table_renders_with_box_drawing_borders() {
        let out = rendered_text("| H1 | H2 |\n|---|---|\n| a | b |");
        assert!(out.contains('┌'), "table should have top-left corner");
        assert!(out.contains('┘'), "table should have bottom-right corner");
        assert!(out.contains('├'), "table should have header separator");
    }

    #[test]
    fn checked_task_renders_x_marker() {
        let out = rendered_text("- [x] done");
        assert!(out.contains("[x] "), "checked task should render '[x] '");
    }

    #[test]
    fn unchecked_task_renders_empty_marker() {
        let out = rendered_text("- [ ] todo");
        assert!(out.contains("[ ] "), "unchecked task should render '[ ] '");
    }

    // Headings

    #[test]
    fn h4_and_deeper_use_triple_hash_prefix() {
        // h4+ all collapse to the h3 style with "### " prefix
        let out = rendered_text("#### Deep");
        assert!(out.contains("### "));
    }

    #[test]
    fn heading_text_appears_after_prefix() {
        let out = rendered_text("# My Title");
        assert!(out.contains("My Title"));
    }

    // Inline formatting

    #[test]
    fn inline_code_appears_in_output() {
        let out = rendered_text("use `foo()` here");
        assert!(out.contains("foo()"));
    }

    #[test]
    fn bold_text_appears_in_output() {
        let out = rendered_text("**important**");
        assert!(out.contains("important"));
    }

    #[test]
    fn italic_text_appears_in_output() {
        let out = rendered_text("_emphasis_");
        assert!(out.contains("emphasis"));
    }

    #[test]
    fn strikethrough_text_appears_in_output() {
        let out = rendered_text("~~deleted~~");
        assert!(out.contains("deleted"));
    }

    // Lists

    #[test]
    fn nested_list_indents_child_items() {
        let out = rendered_text("- parent\n  - child");
        // child item should have extra leading spaces
        let child_pos = out.find("child").unwrap();
        let bullet_pos = out[..child_pos].rfind('•').unwrap();
        let indent = &out[bullet_pos - 2..bullet_pos];
        assert_eq!(indent, "  ", "nested bullet should be indented by 2 spaces");
    }

    #[test]
    fn ordered_list_counter_increments_per_item() {
        let out = rendered_text("1. a\n2. b\n3. c");
        assert!(out.contains("1. "));
        assert!(out.contains("2. "));
        assert!(out.contains("3. "));
    }

    // Code blocks

    #[test]
    fn fenced_code_block_content_appears_in_output() {
        let out = rendered_text("```\nlet x = 1;\n```");
        assert!(out.contains("let x = 1;"));
    }

    #[test]
    fn fenced_code_block_with_language_content_appears_in_output() {
        let out = rendered_text("```rust\nfn main() {}\n```");
        assert!(out.contains("fn main() {}"));
    }

    // Horizontal rule

    #[test]
    fn horizontal_rule_renders_as_line_of_dashes() {
        let out = rendered_text("---");
        assert!(
            out.contains('─'),
            "horizontal rule should render as '─' characters"
        );
    }

    // Table structure

    #[test]
    fn table_header_row_appears_before_separator() {
        let out = rendered_text("| Name | Age |\n|---|---|\n| Alice | 30 |");
        let header_pos = out.find("Name").unwrap();
        let sep_pos = out.find('├').unwrap();
        assert!(
            header_pos < sep_pos,
            "header should appear before ├ separator"
        );
    }

    #[test]
    fn table_body_row_appears_after_separator() {
        let out = rendered_text("| Name | Age |\n|---|---|\n| Alice | 30 |");
        let sep_pos = out.find('├').unwrap();
        let body_pos = out.find("Alice").unwrap();
        assert!(
            body_pos > sep_pos,
            "body row should appear after ├ separator"
        );
    }

    // Blockquote

    #[test]
    fn blockquote_content_appears_after_bar() {
        let out = rendered_text("> hello world");
        let bar_pos = out.find("│ ").unwrap();
        let text_pos = out.find("hello world").unwrap();
        assert!(text_pos > bar_pos);
    }
}
