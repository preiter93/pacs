use crate::{client::PacsClient, theme::Theme};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Borders, List, ListState, Paragraph, StatefulWidget},
};
use tui_world::{Focus, Pointer, keys};
use tui_world::{Keybindings, WidgetId, World};

pub const PROJECTS: WidgetId = WidgetId("Projects");

pub struct Sidebar;

impl Sidebar {
    pub fn render(world: &mut World, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let is_focused = world.get::<Focus>().id == Some(PROJECTS);
        let theme = world.get::<Theme>();

        let block = theme.block_for_focus(is_focused);
        let inner_area = block.inner(area);

        frame.render_widget(block, area);

        world.get_mut::<Pointer>().set(PROJECTS, area);
        world
            .get_mut::<Pointer>()
            .on_click(PROJECTS, |world, _x, _y| {
                world.get_mut::<Focus>().set(PROJECTS);
            });

        Projects::render(world, frame, inner_area);
    }
}

#[derive(Default)]
pub struct ProjectsState {
    pub state: ListState,
}

impl ProjectsState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self { state }
    }

    fn next(&mut self) {
        self.state.select_next();
    }

    fn previous(&mut self) {
        self.state.select_previous();
    }
}

pub struct Projects;

impl Projects {
    pub fn register_keybindings(world: &mut World) {
        let kb = world.get_mut::<Keybindings>();

        kb.bind_many(PROJECTS, keys![KeyCode::Down, 'j'], "Down", |world| {
            world.get_mut::<ProjectsState>().next();
        });

        kb.bind_many(PROJECTS, keys![KeyCode::Up, 'k'], "Up", |world| {
            world.get_mut::<ProjectsState>().previous();
        });
    }

    pub fn render(world: &mut World, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let [project_title_area, projects_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

        let block = theme.block().borders(Borders::BOTTOM);

        let project_title = Paragraph::new(Line::from(vec![
            Span::from(" Projects").style(theme.text_accent),
        ]))
        .block(block);

        frame.render_widget(project_title, project_title_area);

        let projects = client.list_projects().clone();

        let mut state = &mut world.get_mut::<ProjectsState>().state;
        List::new(projects)
            .highlight_symbol("▶ ")
            .render(projects_area, frame.buffer_mut(), state);
    }
}
