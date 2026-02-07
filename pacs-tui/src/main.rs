#![allow(dead_code, unused)]
mod app;
mod client;
mod components;
mod help;
mod sidebar;
mod theme;
mod util;

use ratatui::crossterm::{
    event::{self, Event as CEvent},
    execute,
};
use tui_world::prelude::*;

use crate::{
    app::{GLOBAL, setup_world},
    util::get_active_ids,
};

fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), event::EnableMouseCapture)?;

    let mut world = World::default();
    setup_world(&mut world);

    loop {
        terminal.draw(|frame| app::render(frame, &mut world))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            let active = get_active_ids(&world);

            match event::read()? {
                CEvent::Key(key) => Event::Key(key).handle(&mut world, &active),
                CEvent::Mouse(mouse) => Event::Mouse(mouse).handle(&mut world, &active),
                _ => {}
            }
        }

        if world.get::<app::AppState>().should_quit {
            break;
        }
    }

    execute!(std::io::stdout(), event::DisableMouseCapture)?;
    ratatui::restore();

    Ok(())
}
