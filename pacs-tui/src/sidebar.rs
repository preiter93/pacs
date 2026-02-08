use std::env;

use crate::{client::PacsClient, commands::CommandsState, theme::Theme};
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
    pub num_projects: usize,
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

        Self {
            state,
            num_projects: projects.len(),
        }
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
    fn activate_selected(world: &mut World) {
        let projects = world.get::<PacsClient>().list_projects();
        let selected = world.get::<ProjectsState>().state.selected();
        if let Some(idx) = selected {
            if let Some(name) = projects.get(idx) {
                let _ = world.get_mut::<PacsClient>().set_active_project(name);
                let environments = world.get::<PacsClient>().list_environments();
                let active = world.get::<PacsClient>().active_environment();
                world
                    .get_mut::<EnvironmentsState>()
                    .select_active(&environments, active.as_deref());
                world.get_mut::<CommandsState>().state.select(Some(0));
            }
        }
    }

    pub fn setup_keybindings(world: &mut World) {
        let kb = world.get_mut::<Keybindings>();

        kb.bind(PROJECTS, ' ', "Go to Environments", |world| {
            world.get_mut::<Focus>().set(ENVIRONMENTS);
        });

        kb.bind_many(PROJECTS, keys![KeyCode::Down, 'j'], "Down", |world| {
            world.get_mut::<ProjectsState>().next();
            Projects::activate_selected(world);
        });

        kb.bind_many(PROJECTS, keys![KeyCode::Up, 'k'], "Up", |world| {
            world.get_mut::<ProjectsState>().previous();
            Projects::activate_selected(world);
        });
    }

    pub fn setup_pointer(world: &mut World) {
        world
            .get_mut::<Pointer>()
            .on_click(PROJECTS, |world, area, x, y| {
                if !world.get::<Focus>().is_focused(PROJECTS) {
                    world.get_mut::<Focus>().set(PROJECTS);
                    return;
                }

                let row = (y - area.y) as usize;
                let state = world.get_mut::<ProjectsState>();

                if row >= state.num_projects {
                    return;
                }

                state.state.select(Some(row));
                Projects::activate_selected(world);
            });
    }

    pub fn render(world: &mut World, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let is_focused = world.get::<Focus>().id == Some(PROJECTS);
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let [title_area, content_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

        let block = theme.block().borders(Borders::BOTTOM);

        let title_spans = vec![Span::from(" Projects").style(theme.text_accent)];

        let project_title = Paragraph::new(Line::from(title_spans)).block(block);

        frame.render_widget(project_title, title_area);

        let projects = client.list_projects();

        let items: Vec<Line> = projects
            .iter()
            .map(|name| Line::raw(name.clone()))
            .collect();

        let mut list = List::new(items)
            .highlight_symbol(" > ")
            .highlight_spacing(HighlightSpacing::Always);

        if is_focused {
            list = list.highlight_style(theme.selected);
        }

        let state = &mut world.get_mut::<ProjectsState>().state;
        list.render(content_area, frame.buffer_mut(), state);

        world.get_mut::<Pointer>().set(PROJECTS, content_area);
    }
}

#[derive(Default)]
pub struct EnvironmentsState {
    pub state: ListState,
    pub num_environments: usize,
}

impl EnvironmentsState {
    pub fn new(client: &PacsClient) -> Self {
        let mut s = Self::default();

        let environments = client.list_environments();
        let active = client.active_environment();

        s.select_active(&environments, active.as_deref());

        s
    }

    pub fn select_active(&mut self, environments: &[String], active: Option<&str>) {
        let index = active
            .and_then(|name| environments.iter().position(|e| e == name))
            .unwrap_or(0);

        if !environments.is_empty() {
            self.state.select(Some(index));
        }

        self.num_environments = environments.len();
    }

    fn next(&mut self) {
        self.state.select_next();
    }

    fn previous(&mut self) {
        self.state.select_previous();
    }
}

pub struct Environments;

impl Environments {
    fn activate_selected(world: &mut World) {
        let environments = world.get::<PacsClient>().list_environments();
        let selected = world.get::<EnvironmentsState>().state.selected();
        if let Some(idx) = selected {
            if let Some(name) = environments.get(idx) {
                let _ = world.get_mut::<PacsClient>().set_active_environment(name);
            }
        }
    }

    pub fn setup_keybindings(world: &mut World) {
        let kb = world.get_mut::<Keybindings>();

        kb.bind(ENVIRONMENTS, ' ', "Go to Projects", |world| {
            world.get_mut::<Focus>().set(PROJECTS);
        });

        kb.bind_many(ENVIRONMENTS, keys![KeyCode::Down, 'j'], "Down", |world| {
            world.get_mut::<EnvironmentsState>().next();
            Environments::activate_selected(world);
        });

        kb.bind_many(ENVIRONMENTS, keys![KeyCode::Up, 'k'], "Up", |world| {
            world.get_mut::<EnvironmentsState>().previous();
            Environments::activate_selected(world);
        });
    }

    pub fn setup_pointer(world: &mut World) {
        world
            .get_mut::<Pointer>()
            .on_click(ENVIRONMENTS, |world, area, x, y| {
                if !world.get::<Focus>().is_focused(ENVIRONMENTS) {
                    world.get_mut::<Focus>().set(ENVIRONMENTS);
                    return;
                }

                let row = (y - area.y) as usize;
                let state = world.get_mut::<EnvironmentsState>();

                if row >= state.num_environments {
                    return;
                }

                state.state.select(Some(row));
                Environments::activate_selected(world);
            });
    }

    pub fn render(world: &mut World, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let is_focused = world.get::<Focus>().id == Some(ENVIRONMENTS);
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let [title_area, content_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

        let block = theme.block().borders(Borders::TOP | Borders::BOTTOM);

        let title_spans = vec![Span::from(" Environments").style(theme.text_accent)];

        let env_title = Paragraph::new(Line::from(title_spans)).block(block);

        frame.render_widget(env_title, title_area);

        let environments = client.list_environments();

        let items: Vec<Line> = environments
            .iter()
            .map(|name| Line::raw(name.clone()))
            .collect();

        let mut list = List::new(items)
            .highlight_symbol(" > ")
            .highlight_spacing(HighlightSpacing::Always);

        if is_focused {
            list = list.highlight_style(theme.selected);
        }

        let state = &mut world.get_mut::<EnvironmentsState>().state;
        list.render(content_area, frame.buffer_mut(), state);

        world.get_mut::<Pointer>().set(ENVIRONMENTS, content_area);
    }
}
