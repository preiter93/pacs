#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]

pub mod app;
pub mod client;
pub mod commands;
pub mod components;
pub mod help;
pub mod highlight;
pub mod sidebar;
pub mod theme;
pub mod util;

use ratatui::crossterm::{
    event::{self, Event as CEvent},
    execute,
};
use tui_world::prelude::*;

use crate::{app::setup_world, util::get_active_ids};

/// Run the terminal user interface.
///
/// # Errors
///
/// Returns an error if terminal initialization fails or if there's an I/O error.
pub fn run() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), event::EnableMouseCapture)?;

    let mut world = World::default();
    setup_world(&mut world)?;

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
