use ratatui::style::{Color, Modifier, Style};
use syntect::highlighting::ThemeSet;

#[allow(clippy::struct_field_names)]
pub struct Theme {
    pub h1: Style,
    pub h1_prefix: Style,
    pub h2: Style,
    pub h2_prefix: Style,
    pub h3: Style,
    pub h3_prefix: Style,
    pub bold: Style,
    pub italic: Style,
    pub code_inline: Style,
    pub blockquote: Style,
    pub normal: Style,
    pub counter: Style,
    pub strikethrough: Style,
    pub link: Style,
    pub table_header: Style,
    pub table_border: Style,
    pub syntax_set: syntect::parsing::SyntaxSet,
    pub syntax_highlight_theme: syntect::highlighting::Theme,
}

impl Default for Theme {
    fn default() -> Self {
        let fuchsia = Color::from_u32(0x00EE_6FF8);
        let dim_fuchsia = Color::from_u32(0x0099_519E);
        let dull_fuchsia = Color::from_u32(0x00AD_58B4);
        let green = Color::from_u32(0x0004_B575);
        let dim_green = Color::from_u32(0x0003_6B46);
        let normal_dim = Color::from_u32(0x0077_7777);
        let gray = Color::from_u32(0x0062_6262);
        let syntax_highlight_theme = {
            let theme_bytes = include_bytes!("../assets/github-dark.tmTheme");
            let mut cursor = std::io::Cursor::new(theme_bytes);
            ThemeSet::load_from_reader(&mut cursor)
                .expect("bundled github-dark.tmTheme should always be valid")
        };
        let syntax_set = two_face::syntax::extra_newlines();

        Self {
            h1: Style::default().fg(fuchsia).add_modifier(Modifier::BOLD),
            h1_prefix: Style::default()
                .fg(dim_fuchsia)
                .add_modifier(Modifier::BOLD),
            h2: Style::default().fg(green).add_modifier(Modifier::BOLD),
            h2_prefix: Style::default().fg(dim_green).add_modifier(Modifier::BOLD),
            h3: Style::default()
                .fg(dull_fuchsia)
                .add_modifier(Modifier::BOLD),
            h3_prefix: Style::default()
                .fg(dull_fuchsia)
                .add_modifier(Modifier::DIM),
            bold: Style::default().add_modifier(Modifier::BOLD),
            italic: Style::default().add_modifier(Modifier::ITALIC),
            code_inline: Style::default()
                .fg(Color::from_u32(0x00e1_e4e8))
                .bg(Color::from_u32(0x0024_292e)),
            blockquote: Style::default().fg(normal_dim),
            normal: Style::default(),
            counter: Style::default().fg(gray).add_modifier(Modifier::DIM),
            strikethrough: Style::default().add_modifier(Modifier::CROSSED_OUT),
            link: Style::default()
                .fg(Color::from_u32(0x0058_A6FF))
                .add_modifier(Modifier::UNDERLINED),
            table_header: Style::default().fg(fuchsia).add_modifier(Modifier::BOLD),
            table_border: Style::default().fg(gray),
            syntax_highlight_theme,
            syntax_set,
        }
    }
}
