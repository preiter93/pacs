use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};
use tui_theme_builder::ThemeBuilder;

pub struct Colors {
    pub bg: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub success: Color,
    pub highlight: Color,
    pub surface: Color,
    // Syntax highlighting
    pub syn_string: Color,
    pub syn_flag: Color,
    pub syn_variable: Color,
    pub syn_operator: Color,
    pub syn_comment: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(15, 15, 25),
            fg: Color::Rgb(220, 225, 245),
            muted: Color::Rgb(90, 95, 130),
            accent: Color::Rgb(140, 130, 255),
            accent_secondary: Color::Rgb(80, 180, 255),
            success: Color::Rgb(130, 230, 180),
            highlight: Color::Rgb(45, 40, 80),
            surface: Color::Rgb(25, 25, 40),
            // Syntax highlighting
            syn_string: Color::Rgb(180, 220, 140),
            syn_flag: Color::Rgb(220, 180, 120),
            syn_variable: Color::Rgb(140, 200, 220),
            syn_operator: Color::Rgb(200, 140, 180),
            syn_comment: Color::Rgb(90, 95, 130),
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

    #[style(fg = accent, add_modifier = "Modifier::BOLD")]
    pub text_accent: Style,

    #[style(fg = accent_secondary, add_modifier = "Modifier::BOLD")]
    pub text_accent_alt: Style,

    #[style(fg = accent)]
    pub title: Style,

    #[style(fg = muted)]
    pub border: Style,

    #[style(fg = accent)]
    pub border_focused: Style,

    #[style(fg = fg, bg = highlight)]
    pub selected: Style,

    #[style(fg = accent_secondary, add_modifier = "Modifier::BOLD")]
    pub keybinding_key: Style,

    #[style(fg = success)]
    pub success: Style,

    // Syntax highlighting
    #[style(fg = accent_secondary)]
    pub sh_command: Style,

    #[style(fg = syn_string)]
    pub sh_string: Style,

    #[style(fg = syn_flag)]
    pub sh_flag: Style,

    #[style(fg = syn_variable)]
    pub sh_variable: Style,

    #[style(fg = syn_operator)]
    pub sh_operator: Style,

    #[style(fg = syn_comment)]
    pub sh_comment: Style,

    #[border_type(plain)]
    pub border_type: BorderType,

    #[border_type(thick)]
    pub border_type_focused: BorderType,
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

    pub fn block_for_focus<'a>(&self, focused: bool) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(if focused {
                self.border_type_focused
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
