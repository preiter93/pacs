use ratatui::crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, List, Paragraph, Row, StatefulWidget,
        Table,
    },
};
use tui_world::{Focus, Keybindings, Pointer, WidgetId, World, keys};

use crate::{client::PacsClient, highlight::highlight_shell, theme::Theme};

pub const COMMANDS_LIST: WidgetId = WidgetId("Commands");
pub const COMMANDS_DETAIL: WidgetId = WidgetId("CommandDetail");
pub const COPY_BUTTON: WidgetId = WidgetId("CopyButton");

pub struct CommandsPanel;

impl CommandsPanel {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let focus_id = world.get::<Focus>().id;
        let is_focused = focus_id == Some(COMMANDS_LIST) || focus_id == Some(COMMANDS_DETAIL);
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
    }
}

#[derive(Default)]
pub struct CommandsState {
    pub state: ListState,
    pub num_commands: usize,
}

#[derive(Default)]
pub struct CopyButtonState {
    pub clicked_at: Option<std::time::Instant>,
}

impl CopyButtonState {
    pub fn click(&mut self) {
        self.clicked_at = Some(std::time::Instant::now());
    }

    pub fn is_active(&self) -> bool {
        self.clicked_at
            .map(|t| t.elapsed().as_millis() < 300)
            .unwrap_or(false)
    }
}

impl CommandsState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            num_commands: 0,
        }
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

        kb.bind_many(COMMANDS_LIST, keys![KeyCode::Down, 'j'], "Down", |world| {
            world.get_mut::<CommandsState>().next();
        });

        kb.bind_many(COMMANDS_LIST, keys![KeyCode::Up, 'k'], "Up", |world| {
            world.get_mut::<CommandsState>().previous();
        });

        kb.bind(COMMANDS_LIST, 'c', "Copy", |world| {
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
            .on_click(COMMANDS_LIST, |world, area, _x, y| {
                if !world.get::<Focus>().is_focused(COMMANDS_LIST) {
                    world.get_mut::<Focus>().set(COMMANDS_LIST);
                    return;
                }

                let row = (y - area.y) as usize;
                let state = world.get_mut::<CommandsState>();

                if row >= state.num_commands {
                    return;
                }

                state.state.select(Some(row));
            });

        world
            .get_mut::<Pointer>()
            .on_click(COMMANDS_DETAIL, |world, _, _x, _y| {
                world.get_mut::<Focus>().set(COMMANDS_LIST);
            });

        world
            .get_mut::<Pointer>()
            .on_click(COPY_BUTTON, |world, _, _x, _y| {
                let commands = world.get::<PacsClient>().list_commands();
                let selected = world.get::<CommandsState>().state.selected();
                if let Some(idx) = selected {
                    if let Some(cmd) = commands.get(idx) {
                        let _ = world.get_mut::<PacsClient>().copy_command(&cmd.name);
                        world.get_mut::<CopyButtonState>().click();
                    }
                }
            });
    }

    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let is_focused = world.get::<Focus>().id == Some(COMMANDS_LIST);
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
        let num_commands = commands.len();

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

        let state = world.get_mut::<CommandsState>();
        state.num_commands = num_commands;
        list.render(commands_area, frame.buffer_mut(), &mut state.state);

        world.get_mut::<Pointer>().set(COMMANDS_LIST, commands_area);
    }
}

pub struct CommandDetail;

impl CommandDetail {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();
        let selected = world.get::<CommandsState>().state.selected();
        let button_active = world.get::<CopyButtonState>().is_active();

        let block = theme.block().borders(Borders::LEFT);
        frame.render_widget(block.clone(), area);

        let inner = block.inner(area);

        let [content_area, button_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(inner);

        let Some(cmd) = selected.and_then(|i| client.list_commands().get(i).cloned()) else {
            return;
        };

        let lines = highlight_shell(&cmd.command, theme);
        let content =
            Paragraph::new(Text::from(lines)).wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(content, content_area);

        // Copy button
        let (button_text, button_style, show_hint) = if button_active {
            (" Copied! ", theme.success, false)
        } else {
            (" Copy ", theme.text_accent, true)
        };
        let button_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if button_active {
                theme.success
            } else {
                theme.border
            })
            .border_type(BorderType::Rounded);
        let mut button_spans = vec![Span::styled(button_text, button_style)];
        if show_hint {
            button_spans.push(Span::styled("[c]", theme.text_muted));
        }
        let button = Paragraph::new(Line::from(button_spans)).block(button_block);
        frame.render_widget(button, button_area);

        world
            .get_mut::<Pointer>()
            .set(COMMANDS_DETAIL, content_area);
        world.get_mut::<Pointer>().set(COPY_BUTTON, button_area);
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
            .map(|(k, v)| {
                Row::new(vec![
                    Cell::new(k.clone()).style(theme.text_muted),
                    Cell::new(v.clone()).style(theme.text_accent_alt),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        );
        frame.render_widget(table, block.inner(area));
    }
}
