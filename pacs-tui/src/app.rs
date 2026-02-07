use crate::{
    client::PacsClient,
    commands::{CONTENT, Commands, CommandsPanel, CommandsState},
    components::selectable_text::Selections,
    help,
    sidebar::{
        ENVIRONMENTS, Environments, EnvironmentsState, PROJECTS, Projects, ProjectsState, Sidebar,
    },
    util::kc,
};
use anyhow::Result;
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

/// Focus ring order for Tab navigation
const FOCUS_RING: [WidgetId; 2] = [PROJECTS, CONTENT];

#[derive(Default)]
pub struct AppState {
    pub should_quit: bool,
    pub help_open: bool,
    pub area: Rect,
}

pub fn setup_world(world: &mut World) -> Result<()> {
    world.insert(Theme::default());
    world.insert(AppState::default());
    world.insert(Focus::new(PROJECTS));
    let client = PacsClient::new()?;
    world.insert(ProjectsState::new(&client));
    world.insert(EnvironmentsState::new(&client));
    world.insert(CommandsState::new());
    world.insert(client);

    global_keybindings(world);
    Projects::register_keybindings(world);
    Environments::register_keybindings(world);
    Commands::register_keybindings(world);

    Ok(())
}

fn global_keybindings(world: &mut World) {
    let kb = world.get_mut::<Keybindings>();

    kb.bind(GLOBAL, KeyBinding::ctrl('c'), "Quit", |world| {
        world.get_mut::<AppState>().should_quit = true;
    });

    kb.bind(GLOBAL, '?', "Help", |world| {
        help::toggle(world);
    });

    kb.bind(GLOBAL, KeyCode::Tab, "Next Focus", |world| {
        let focus = world.get_mut::<Focus>();
        if let Some(current) = focus.id {
            let current = if current == ENVIRONMENTS {
                PROJECTS
            } else {
                current
            };
            if let Some(idx) = FOCUS_RING.iter().position(|&id| id == current) {
                let next = (idx + 1) % FOCUS_RING.len();
                focus.id = Some(FOCUS_RING[next]);
            }
        }
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
    let [header, content] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

    render_title(world, frame, header);
    render_content(world, frame, content);
}

pub fn render_content(world: &mut World, frame: &mut Frame, area: Rect) {
    let [sidebar, main] =
        Layout::horizontal([Constraint::Percentage(20), Constraint::Min(0)]).areas(area);

    Sidebar::render(world, frame, sidebar);
    CommandsPanel::render(world, frame, main);
}

fn render_title(world: &mut World, frame: &mut ratatui::Frame, area: Rect) {
    let theme = world.get::<Theme>();

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" PACS", theme.text_accent),
        Span::styled(" - ", theme.text_muted),
        Span::styled("Project Aware Command Storage", theme.text_muted),
    ]));

    frame.render_widget(title, area);

    // Render a subtle separator line below
    let separator = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.border)
        .border_type(BorderType::Rounded);

    frame.render_widget(separator, area);
}
