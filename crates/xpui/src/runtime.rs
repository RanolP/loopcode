use crate::{FocusId, Node};

pub trait UiApp {
    fn render(&mut self) -> Node;

    fn on_input(&mut self, _event: UiInputEvent) {}

    fn focus_state(&mut self) -> Option<&mut FocusState> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiKeyInput {
    Tab,
    BackTab,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Esc,
    Char(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputEvent {
    Key(UiKeyInput),
    ScrollLines(i16),
}

#[derive(Clone, Debug, Default)]
pub struct FocusState {
    focused: Option<FocusId>,
}

impl FocusState {
    pub fn focused(&self) -> Option<FocusId> {
        self.focused
    }

    pub fn is_focused(&self, id: FocusId) -> bool {
        self.focused == Some(id)
    }

    pub fn set_focused(&mut self, id: FocusId) {
        self.focused = Some(id);
    }

    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    pub fn ensure_valid(&mut self, order: &[FocusId]) {
        if order.is_empty() {
            self.focused = None;
            return;
        }

        if self
            .focused
            .is_none_or(|focused| !order.iter().any(|id| *id == focused))
        {
            self.focused = order.first().copied();
        }
    }

    pub fn focus_next(&mut self, order: &[FocusId]) {
        if order.is_empty() {
            self.focused = None;
            return;
        }

        let next = match self.focused {
            Some(current) => {
                let idx = order.iter().position(|id| *id == current).unwrap_or(0);
                order[(idx + 1) % order.len()]
            }
            None => order[0],
        };
        self.focused = Some(next);
    }

    pub fn focus_prev(&mut self, order: &[FocusId]) {
        if order.is_empty() {
            self.focused = None;
            return;
        }

        let prev = match self.focused {
            Some(current) => {
                let idx = order.iter().position(|id| *id == current).unwrap_or(0);
                order[(idx + order.len() - 1) % order.len()]
            }
            None => order[0],
        };
        self.focused = Some(prev);
    }
}

#[derive(Clone, Debug)]
pub struct FocusListState {
    item_heights: Vec<u16>,
    viewport_lines: u16,
    gap_lines: u16,
    focused_index: u16,
    scroll_offset: u16,
}

impl FocusListState {
    pub fn new(item_heights: Vec<u16>, viewport_lines: u16, gap_lines: u16) -> Self {
        Self {
            item_heights,
            viewport_lines: viewport_lines.max(1),
            gap_lines,
            focused_index: 0,
            scroll_offset: 0,
        }
    }

    pub fn focused_index(&self) -> u16 {
        self.focused_index
    }

    pub fn scroll_offset(&self) -> u16 {
        self.scroll_offset
    }

    pub fn viewport_lines(&self) -> u16 {
        self.viewport_lines
    }

    pub fn item_count(&self) -> u16 {
        self.item_heights.len() as u16
    }

    pub fn max_scroll_offset(&self) -> u16 {
        self.content_lines().saturating_sub(self.viewport_lines)
    }

    pub fn content_lines(&self) -> u16 {
        let mut lines = 0u16;
        for (i, height) in self.item_heights.iter().copied().enumerate() {
            lines = lines.saturating_add(height);
            if i + 1 < self.item_heights.len() {
                lines = lines.saturating_add(self.gap_lines);
            }
        }
        lines
    }

    pub fn item_height(&self, index: u16) -> u16 {
        self.item_heights
            .get(index as usize)
            .copied()
            .unwrap_or(1)
            .max(1)
    }

    pub fn item_top_line(&self, index: u16) -> u16 {
        let mut top = 0u16;
        for i in 0..index.min(self.item_count()) {
            top = top
                .saturating_add(self.item_height(i))
                .saturating_add(self.gap_lines);
        }
        top
    }

    pub fn set_focused_index(&mut self, index: u16) {
        self.focused_index = index.min(self.item_count().saturating_sub(1));
        self.ensure_focused_visible();
    }

    pub fn move_focus_by(&mut self, delta: i16) {
        let current = self.focused_index as i16;
        let next = (current + delta).clamp(0, self.item_count().saturating_sub(1) as i16) as u16;
        self.set_focused_index(next);
    }

    pub fn ensure_focused_visible(&mut self) {
        let top = self.item_top_line(self.focused_index);
        let height = self.item_height(self.focused_index);
        let bottom = top.saturating_add(height);

        if top < self.scroll_offset {
            self.scroll_offset = top;
        } else {
            let viewport_end = self.scroll_offset.saturating_add(self.viewport_lines);
            if bottom > viewport_end {
                if height >= self.viewport_lines {
                    self.scroll_offset = top;
                } else {
                    self.scroll_offset = bottom.saturating_sub(self.viewport_lines);
                }
            }
        }

        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 80.0,
            height: 24.0,
        }
    }
}

#[cfg(feature = "backend-cpui")]
pub fn run_cpui<A: UiApp + 'static>(app: A) {
    crate::backends::run_cpui(app, WindowSize::default())
}

#[cfg(feature = "backend-cpui")]
pub fn run_cpui_with_size<A: UiApp + 'static>(app: A, size: WindowSize) {
    crate::backends::run_cpui(app, size)
}

pub fn run_gpui<A: UiApp + 'static>(app: A) {
    crate::backends::run_gpui(app, WindowSize::default())
}

pub fn run_gpui_with_size<A: UiApp + 'static>(app: A, size: WindowSize) {
    crate::backends::run_gpui(app, size)
}
