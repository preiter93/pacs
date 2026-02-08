use crate::app::{AppState, GLOBAL};
use crate::theme::Theme;
use crate::util::get_active_ids;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::collections::BTreeMap;
use tui_world::prelude::*;

const BACKDROP: WidgetId = WidgetId("backdrop");

pub fn toggle(world: &mut World) {
    let is_open = world.get::<AppState>().help_open;
    if is_open {
        close(world);
    } else {
        open(world);
    }
}

pub fn open(world: &mut World) {
    world.get_mut::<AppState>().help_open = true;

    let area = world.get::<AppState>().area;
    let dialog_area = center_rect(area, 40, 15);

    world.get_mut::<Pointer>().set(BACKDROP, area);
    world
        .get_mut::<Pointer>()
        .on_click(BACKDROP, move |world, _, x, y| {
            if !dialog_area.contains((x, y).into()) {
                close(world);
            }
        });
}

pub fn close(world: &mut World) {
    world.get_mut::<AppState>().help_open = false;
    world.get_mut::<Pointer>().remove(BACKDROP);
}

pub fn render(world: &World, frame: &mut Frame, area: Rect) {
    let theme = world.get::<Theme>();
    let dialog_area = center_rect(area, 40, 15);

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_style(theme.border);

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let active = get_active_ids(&world);

    let keybindings = world.get::<Keybindings>();
    let display = keybindings.display_for(&active);

    let mut lines: Vec<Line> = Vec::new();

    let mut groups: BTreeMap<WidgetId, BTreeMap<&'static str, Vec<&DisplayInfo>>> = BTreeMap::new();
    for info in &display {
        groups
            .entry(info.id)
            .or_default()
            .entry(info.name)
            .or_default()
            .push(info);
    }

    for (id, commands) in groups {
        let header = format!("[{}]", id.0);

        if !lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines.push(Line::from(Span::styled(header, theme.title)));

        for (name, infos) in commands {
            let keys = infos
                .iter()
                .map(|i| i.key.to_string())
                .collect::<Vec<_>>()
                .join("/");

            lines.push(Line::from(vec![
                Span::styled(format!("{:>12}", keys), theme.keybinding_key),
                Span::raw("  "),
                Span::styled(name, theme.text),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(4));

    let [_, h_center, _] = ratatui::layout::Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .areas(area);

    let [_, dialog, _] = ratatui::layout::Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .areas(h_center);

    dialog
}
