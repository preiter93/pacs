use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
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

    #[style(fg = muted)]
    pub border: Style,

    #[style(fg = accent)]
    pub border_focused: Style,

    #[style(fg = fg, bg = highlight)]
    pub selected: Style,

    #[style(fg = success)]
    pub keybinding_key: Style,

    #[border_type(plain)]
    pub border_type: BorderType,
}

impl Default for Theme {
    fn default() -> Self {
        Self::build(&Colors::default())
    }
}

impl Theme {
    pub fn block<'a>(&self) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(self.border_type)
            .border_style(self.border)
    }

    /// Get a block styled based on focus state
    pub fn block_for_focus<'a>(&self, focused: bool) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(if focused {
                BorderType::Thick
            } else {
                self.border_type
            })
            .border_style(if focused {
                self.border_focused
            } else {
                self.border
            })
    }
}
