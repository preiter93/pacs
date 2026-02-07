use ratatui::{
    style::{Color, Style},
    widgets::BorderType,
};
use tui_theme_builder::ThemeBuilder;

pub struct Colors {
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub success: Color,
    pub highlight: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            fg: Color::Rgb(192, 202, 245),
            muted: Color::Rgb(86, 95, 137),
            accent: Color::Rgb(122, 162, 247),
            success: Color::Rgb(158, 206, 106),
            highlight: Color::Rgb(51, 70, 124),
        }
    }
}

#[derive(ThemeBuilder)]
#[builder(context = Colors)]
pub struct Theme {
    #[style(fg = fg)]
    pub text: Style,

    #[style(fg = muted)]
    pub text_muted: Style,

    #[style(fg = accent, bold)]
    pub text_accent: Style,

    #[style(fg = accent)]
    pub title: Style,

    #[style(fg = accent)]
    pub border: Style,

    #[style(fg = accent)]
    pub border_focused: Style,

    #[style(fg = fg, bg = highlight)]
    pub selected: Style,

    #[style(fg = success)]
    pub keybinding_key: Style,

    #[border_type(rounded)]
    pub border_type: BorderType,
}

impl Default for Theme {
    fn default() -> Self {
        Self::build(&Colors::default())
    }
}
