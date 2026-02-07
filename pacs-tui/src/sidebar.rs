use crate::{client::PacsClient, theme::Theme};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Borders, HighlightSpacing, List, ListState, Paragraph, StatefulWidget},
};
use tui_world::{Focus, Pointer, keys};
use tui_world::{Keybindings, WidgetId, World};

pub const PROJECTS: WidgetId = WidgetId("Projects");
pub const ENVIRONMENTS: WidgetId = WidgetId("Environments");

pub struct Sidebar;

impl Sidebar {
    pub fn render(world: &mut World, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let focus_id = world.get::<Focus>().id;
        let is_focused = focus_id == Some(PROJECTS) || focus_id == Some(ENVIRONMENTS);
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

        let [projects_area, environments_area] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(inner_area);

        Projects::render(world, frame, projects_area);
        Environments::render(world, frame, environments_area);
    }
}

#[derive(Default)]
pub struct ProjectsState {
    pub state: ListState,
}

impl ProjectsState {
    pub fn new(client: &PacsClient) -> Self {
        let mut state = ListState::default();

        let projects = client.list_projects();
        let active = client.active_project();

        let index = active
            .and_then(|name| projects.iter().position(|p| p == &name))
            .unwrap_or(0);

        state.select(Some(index));

        Self { state }
    }

    fn next(&mut self) {
        self.state.select_next();
    }

    fn previous(&mut self) {
        self.state.select_previous();
    }
}

#[derive(Default)]
pub struct EnvironmentsState {
    pub state: ListState,
}

impl EnvironmentsState {
    pub fn new(client: &PacsClient) -> Self {
        let mut state = ListState::default();

        let environments = client.list_environments();
        let active = client.active_environment();

        let index = active
            .and_then(|name| environments.iter().position(|e| e == &name))
            .unwrap_or(0);

        if !environments.is_empty() {
            state.select(Some(index));
        }

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

        kb.bind(PROJECTS, 'e', "Environments", |world| {
            world.get_mut::<Focus>().set(ENVIRONMENTS);
        });

        kb.bind_many(PROJECTS, keys![KeyCode::Down, 'j'], "Down", |world| {
            world.get_mut::<ProjectsState>().next();
        });

        kb.bind_many(PROJECTS, keys![KeyCode::Up, 'k'], "Up", |world| {
            world.get_mut::<ProjectsState>().previous();
        });
    }

    pub fn render(world: &mut World, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let is_focused = world.get::<Focus>().id == Some(PROJECTS);
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let [project_title_area, projects_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

        let block = theme.block().borders(Borders::BOTTOM);

        let title_spans = vec![
            Span::from(" Projects").style(theme.text_accent),
            Span::from(" [p]").style(theme.text_muted),
        ];

        let project_title = Paragraph::new(Line::from(title_spans)).block(block);

        frame.render_widget(project_title, project_title_area);

        let projects = client.list_projects();

        let mut list = List::new(projects)
            .highlight_symbol(" > ")
            .highlight_spacing(HighlightSpacing::Always);

        if is_focused {
            list = list.highlight_style(theme.selected);
        }

        let state = &mut world.get_mut::<ProjectsState>().state;
        list.render(projects_area, frame.buffer_mut(), state);
    }
}

pub struct Environments;

impl Environments {
    pub fn register_keybindings(world: &mut World) {
        let kb = world.get_mut::<Keybindings>();

        kb.bind(ENVIRONMENTS, 'p', "Projects", |world| {
            world.get_mut::<Focus>().set(PROJECTS);
        });

        kb.bind_many(ENVIRONMENTS, keys![KeyCode::Down, 'j'], "Down", |world| {
            world.get_mut::<EnvironmentsState>().next();
        });

        kb.bind_many(ENVIRONMENTS, keys![KeyCode::Up, 'k'], "Up", |world| {
            world.get_mut::<EnvironmentsState>().previous();
        });
    }

    pub fn render(world: &mut World, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let is_focused = world.get::<Focus>().id == Some(ENVIRONMENTS);
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let [env_title_area, environments_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

        let block = theme.block().borders(Borders::TOP | Borders::BOTTOM);

        let title_spans = vec![
            Span::from(" Environments").style(theme.text_accent),
            Span::from(" [e]").style(theme.text_muted),
        ];

        let env_title = Paragraph::new(Line::from(title_spans)).block(block);

        frame.render_widget(env_title, env_title_area);

        let environments = client.list_environments();

        let mut list = List::new(environments)
            .highlight_symbol(" > ")
            .highlight_spacing(HighlightSpacing::Always);

        if is_focused {
            list = list.highlight_style(theme.selected);
        }

        let state = &mut world.get_mut::<EnvironmentsState>().state;
        list.render(environments_area, frame.buffer_mut(), state);
    }
}
