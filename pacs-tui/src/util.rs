use ratatui::crossterm::event::KeyCode;
use tui_world::{Focus, WidgetId, World};

use crate::app::GLOBAL;

pub fn kc(c: char) -> KeyCode {
    KeyCode::Char(c)
}

pub fn get_active_ids(world: &World) -> Vec<WidgetId> {
    let mut active = vec![GLOBAL];

    if let Some(id) = world.get::<Focus>().id {
        active.push(id);
    }

    active
}
