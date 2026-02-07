use crate::{components::selectable_text::Selections, help, util::kc};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tui_world::{Focus, KeyBinding, Keybindings, Pointer, WidgetId, World};

use crate::theme::Theme;

pub const GLOBAL: WidgetId = WidgetId("Global");

#[derive(Default)]
pub struct AppState {
    pub should_quit: bool,
    pub help_open: bool,
    pub area: Rect,
}

pub fn setup(world: &mut World) {
    world.insert(Theme::default());
    world.insert(AppState::default());
    world.insert(Focus::default());
    world.insert(Selections::default());

    global_keybindings(world);
}

fn global_keybindings(world: &mut World) {
    let kb = world.get_mut::<Keybindings>();

    kb.bind(GLOBAL, KeyBinding::ctrl('c'), "Quit", |world| {
        world.get_mut::<AppState>().should_quit = true;
    });

    kb.bind(GLOBAL, KeyBinding::key(kc('?')), "Help", |world| {
        help::toggle(world);
    });
}

pub fn render(frame: &mut Frame, world: &mut World) {
    let area = frame.area();
    world.get_mut::<AppState>().area = area;

    render_main(world, frame, area);

    if world.get::<AppState>().help_open {
        help::render(world, frame, area);
    }
}

pub fn render_main(world: &mut World, frame: &mut Frame, area: Rect) {
    let [header, _] = Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

    render_title(world, frame, header);
}

fn render_title(world: &mut World, frame: &mut ratatui::Frame, area: Rect) {
    let theme = world.get::<Theme>();

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.border_focused)
        .border_type(BorderType::Thick);

    let title = Paragraph::new(Line::from(vec![
        Span::from("PACS").style(theme.text_accent),
        Span::from(" - Project Aware Command Storage").style(theme.text_muted),
    ]))
    .block(block);

    frame.render_widget(title, area);

    // SelectableText::new(TEXT_ID, "PACS")
    //     .style(theme.text)
    //     .selection_style(theme.text_muted)
    //     .render(area, frame.buffer_mut(), world);
}
