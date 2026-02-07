use ratatui::{Frame, layout::Rect};
use tui_world::{Focus, WidgetId, World};

use crate::theme::Theme;

pub const CONTENT: WidgetId = WidgetId("Content");

pub struct MainPanel;

impl MainPanel {
    pub fn render(world: &mut World, frame: &mut Frame, area: Rect) {
        let is_focused = world.get::<Focus>().id == Some(CONTENT);
        let theme = world.get::<Theme>();

        let block = theme.block_for_focus(is_focused).title(" Content ");
        frame.render_widget(block, area);
    }
}
