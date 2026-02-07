use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Borders, Cell, Paragraph, Row, Table},
};
use tui_world::{Focus, Pointer, WidgetId, World};

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

        let [commands_area, environment_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(5)]).areas(inner_area);

        Commands::render(world, frame, commands_area);
        EnvironmentPanel::render(world, frame, environment_area);

        world.get_mut::<Pointer>().set(CONTENT, area);
        world
            .get_mut::<Pointer>()
            .on_click(CONTENT, |world, _x, _y| {
                world.get_mut::<Focus>().set(CONTENT);
            });
    }
}

pub struct Commands;

impl Commands {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let theme = world.get::<Theme>();

        let [title_area, _commands_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

        let block = theme.block().borders(Borders::BOTTOM);

        let title = Paragraph::new(Line::from(vec![
            Span::from(" Commands").style(theme.text_accent),
        ]))
        .block(block);

        frame.render_widget(title, title_area);
    }
}

pub struct EnvironmentPanel;

impl EnvironmentPanel {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let theme = world.get::<Theme>();
        let client = world.get::<PacsClient>();

        let [title_area, table_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

        let block = theme.block().borders(Borders::TOP | Borders::BOTTOM);

        let title = Paragraph::new(Line::from(vec![
            Span::from(" Environment").style(theme.text_accent),
        ]))
        .block(block);

        frame.render_widget(title, title_area);

        let values = client.environment_values();
        let rows: Vec<Row> = values
            .iter()
            .map(|(k, v)| Row::new(vec![Cell::new(k.clone()), Cell::new(v.clone())]))
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        )
        .column_spacing(1);

        frame.render_widget(table, table_area);
    }
}
