use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::{Pointer, WidgetId, World};
use std::collections::HashMap;

/// Stores selection state for all SelectableText widgets.
/// Add to World with `world.insert(Selections::default())`.
#[derive(Default)]
pub struct Selections {
    states: HashMap<WidgetId, SelectionState>,
}

#[derive(Default, Clone)]
struct SelectionState {
    start: Option<usize>,
    end: Option<usize>,
    area: Rect,
    registered: bool,
}

impl SelectionState {
    fn selection(&self) -> Option<(usize, usize)> {
        let (s, e) = (self.start?, self.end?);
        if s == e {
            return None;
        }
        Some((s.min(e), s.max(e)))
    }

    fn coords_to_index(&self, x: u16, y: u16) -> usize {
        if self.area.width == 0 {
            return 0;
        }
        let rel_x = x.saturating_sub(self.area.x) as usize;
        let rel_y = y.saturating_sub(self.area.y) as usize;
        rel_y * self.area.width as usize + rel_x
    }
}

impl Selections {
    /// Get the selected text for a widget.
    pub fn selected_text<'a>(&self, id: WidgetId, text: &'a str) -> Option<&'a str> {
        let state = self.states.get(&id)?;
        let (start, end) = state.selection()?;
        text.get(start..end.min(text.len()))
    }

    /// Clear selection for a widget.
    pub fn clear(&mut self, id: WidgetId) {
        if let Some(state) = self.states.get_mut(&id) {
            state.start = None;
            state.end = None;
        }
    }

    pub fn get_selection(&self, id: WidgetId) -> Option<(usize, usize)> {
        self.states.get(&id)?.selection()
    }
}

/// A selectable text widget.
///
/// # Example
///
/// ```ignore
/// // Setup (once):
/// world.insert(Selections::default());
///
/// // In render:
/// SelectableText::new(TEXT_ID, "Click and drag to select!")
///     .style(Style::default().fg(Color::White))
///     .selection_style(Style::default().bg(Color::Blue))
///     .render(area, frame.buffer_mut(), &mut world);
///
/// // Get selected text:
/// let selected = world.get::<Selections>().selected_text(TEXT_ID, "...");
/// ```
pub struct SelectableText<'a> {
    id: WidgetId,
    text: &'a str,
    style: Style,
    selection_style: Style,
}

impl<'a> SelectableText<'a> {
    pub fn new(id: WidgetId, text: &'a str) -> Self {
        Self {
            id,
            text,
            style: Style::default(),
            selection_style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn selection_style(mut self, style: Style) -> Self {
        self.selection_style = style;
        self
    }

    /// Render the widget. Automatically registers pointer handlers.
    pub fn render(self, area: Rect, buf: &mut Buffer, world: &mut World) {
        // Get pending registration info first
        let needs_register = {
            let selections = world.get_mut::<Selections>();
            let state = selections.states.entry(self.id).or_default();
            let needs = !state.registered;
            if needs {
                state.registered = true;
            }
            state.area = area;
            needs
        };

        // Register handlers if needed
        if needs_register {
            let id = self.id;
            let pointer = world.get_mut::<Pointer>();

            pointer.on_down(id, move |w, x, y| {
                if let Some(state) = w.get_mut::<Selections>().states.get_mut(&id) {
                    let idx = state.coords_to_index(x, y);
                    state.start = Some(idx);
                    state.end = Some(idx);
                }
            });

            pointer.on_drag(id, move |w, x, y| {
                if let Some(state) = w.get_mut::<Selections>().states.get_mut(&id) {
                    if state.start.is_some() {
                        state.end = Some(state.coords_to_index(x, y));
                    }
                }
            });
        }

        // Set hit area
        world.get_mut::<Pointer>().set(
            self.id,
            crate::Area::new(area.x, area.y, area.width, area.height),
        );

        // Render with selection highlighting
        let selection = world.get::<Selections>().get_selection(self.id);
        let text_len = self.text.len();

        let spans: Vec<Span> = self
            .text
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let is_selected = selection
                    .map(|(s, e)| i >= s && i < e.min(text_len))
                    .unwrap_or(false);
                let style = if is_selected {
                    self.selection_style
                } else {
                    self.style
                };
                Span::styled(c.to_string(), style)
            })
            .collect();

        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}
