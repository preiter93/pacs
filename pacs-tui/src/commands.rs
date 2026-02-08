use ratatui::crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Borders, Cell, HighlightSpacing, List, Paragraph, Row, StatefulWidget, Table},
};
use tui_world::{Focus, Keybindings, Pointer, WidgetId, World, keys};

use crate::{client::PacsClient, theme::Theme};

pub const CONTENT: WidgetId = WidgetId("Commands");

pub struct CommandsPanel;

impl CommandsPanel {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let is_focused = world.get::<Focus>().id == Some(CONTENT);
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let block = theme.block_for_focus(is_focused);
        let inner_area = block.inner(area);

        frame.render_widget(block, area);

        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(5)]).areas(inner_area);

        let [commands_area, detail_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(main_area);

        Commands::render(world, frame, commands_area);
        CommandDetail::render(world, frame, detail_area);
        BottomPanel::render(world, frame, bottom_area);

        world.get_mut::<Pointer>().set(CONTENT, area);
    }
}

#[derive(Default)]
pub struct CommandsState {
    pub state: ListState,
}

impl CommandsState {
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

pub struct Commands;

impl Commands {
    pub fn setup_keybindings(world: &mut World) {
        let kb = world.get_mut::<Keybindings>();

        kb.bind_many(CONTENT, keys![KeyCode::Down, 'j'], "Down", |world| {
            world.get_mut::<CommandsState>().next();
        });

        kb.bind_many(CONTENT, keys![KeyCode::Up, 'k'], "Up", |world| {
            world.get_mut::<CommandsState>().previous();
        });

        kb.bind(CONTENT, 'c', "Copy", |world| {
            let commands = world.get::<PacsClient>().list_commands();
            let selected = world.get::<CommandsState>().state.selected();
            if let Some(idx) = selected {
                if let Some(cmd) = commands.get(idx) {
                    let _ = world.get_mut::<PacsClient>().copy_command(&cmd.name);
                }
            }
        });
    }

    pub fn setup_pointer(world: &mut World) {
        world
            .get_mut::<Pointer>()
            .on_click(CONTENT, |world, _, _x, _y| {
                world.get_mut::<Focus>().set(CONTENT);
            });
    }

    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let is_focused = world.get::<Focus>().id == Some(CONTENT);
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let [title_area, commands_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

        let block = theme.block().borders(Borders::BOTTOM);

        let title = Paragraph::new(Line::from(vec![
            Span::from(" Commands").style(theme.text_accent),
        ]))
        .block(block);

        frame.render_widget(title, title_area);

        let commands = client.list_commands();
        let items: Vec<Line> = commands
            .iter()
            .map(|cmd| Line::raw(cmd.name.clone()))
            .collect();

        let mut list = List::new(items)
            .highlight_symbol(" > ")
            .highlight_spacing(HighlightSpacing::Always);

        if is_focused {
            list = list.highlight_style(theme.selected);
        }

        let state = &mut world.get_mut::<CommandsState>().state;
        list.render(commands_area, frame.buffer_mut(), state);
    }
}

pub struct CommandDetail;

impl CommandDetail {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();
        let selected = world.get::<CommandsState>().state.selected();

        let block = theme.block().borders(Borders::LEFT);
        frame.render_widget(block.clone(), area);

        let Some(cmd) = selected.and_then(|i| client.list_commands().get(i).cloned()) else {
            return;
        };

        let content = Paragraph::new(cmd.command).wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(content, block.inner(area));
    }
}

pub struct BottomPanel;

impl BottomPanel {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let block = theme.block().borders(Borders::TOP);
        frame.render_widget(block.clone(), area);

        let rows: Vec<Row> = client
            .environment_values()
            .iter()
            .map(|(k, v)| Row::new(vec![Cell::new(k.clone()), Cell::new(v.clone())]))
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        );
        frame.render_widget(table, block.inner(area));
    }
}
