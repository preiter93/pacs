use ratatui::{Frame, layout::Rect};
use tui_world::{Focus, Pointer, WidgetId, World};

use crate::theme::Theme;

pub const CONTENT: WidgetId = WidgetId("Content");

pub struct MainPanel;

impl MainPanel {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let is_focused = world.get::<Focus>().id == Some(CONTENT);
        let theme = world.get::<Theme>();

        let block = theme.block_for_focus(is_focused);
        frame.render_widget(block, area);

        world.get_mut::<Pointer>().set(CONTENT, area);
        world
            .get_mut::<Pointer>()
            .on_click(CONTENT, |world, _x, _y| {
                world.get_mut::<Focus>().set(CONTENT);
            });
    }
}
