use crate::FocusId;

use super::{FocusState, UiInputEvent, UiKeyInput};

#[derive(Clone, Debug)]
pub struct FocusListState {
    item_heights: Vec<u16>,
    viewport_lines: u16,
    gap_lines: u16,
    focused_index: u16,
    scroll_offset: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct FocusListBinding {
    first_focus_id: u64,
}

impl FocusListBinding {
    pub fn new(first_focus_id: u64) -> Self {
        Self { first_focus_id }
    }

    pub fn focus_id(&self, index: u16) -> FocusId {
        FocusId(self.first_focus_id + index as u64)
    }

    pub fn focused_index(&self, focus: &FocusState, item_count: u16) -> Option<u16> {
        let id = focus.focused()?.0;
        let end = self.first_focus_id + item_count as u64;
        if (self.first_focus_id..end).contains(&id) {
            Some((id - self.first_focus_id) as u16)
        } else {
            None
        }
    }

    pub fn sync_list_from_focus(&self, focus: &FocusState, list: &mut FocusListState) {
        if let Some(index) = self.focused_index(focus, list.item_count()) {
            list.set_focused_index(index);
        }
    }

    pub fn handle_input(
        &self,
        focus: &mut FocusState,
        list: &mut FocusListState,
        event: UiInputEvent,
    ) -> bool {
        let Some(index) = self.focused_index(focus, list.item_count()) else {
            return false;
        };
        list.set_focused_index(index);

        let handled = match event {
            UiInputEvent::Key(UiKeyInput::Up) => {
                list.move_focus_by(-1);
                true
            }
            UiInputEvent::Key(UiKeyInput::Down) => {
                list.move_focus_by(1);
                true
            }
            UiInputEvent::Key(UiKeyInput::PageUp) => {
                list.move_focus_by(-(list.viewport_lines() as i16));
                true
            }
            UiInputEvent::Key(UiKeyInput::PageDown) => {
                list.move_focus_by(list.viewport_lines() as i16);
                true
            }
            UiInputEvent::Key(UiKeyInput::Home) => {
                list.set_focused_index(0);
                true
            }
            UiInputEvent::Key(UiKeyInput::End) => {
                list.set_focused_index(list.item_count().saturating_sub(1));
                true
            }
            UiInputEvent::Key(UiKeyInput::Tab | UiKeyInput::BackTab) => {
                list.ensure_focused_visible();
                true
            }
            UiInputEvent::ScrollLines(lines) if lines < 0 => {
                list.move_focus_by(-(lines.unsigned_abs() as i16));
                true
            }
            UiInputEvent::ScrollLines(lines) if lines > 0 => {
                list.move_focus_by(lines);
                true
            }
            _ => false,
        };

        if handled {
            focus.set_focused(self.focus_id(list.focused_index()));
        }
        handled
    }
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

    pub fn set_viewport_lines(&mut self, viewport_lines: u16) {
        self.viewport_lines = viewport_lines.max(1);
        self.ensure_focused_visible();
    }

    pub fn item_count(&self) -> u16 {
        self.item_heights.len() as u16
    }

    pub fn set_item_heights(&mut self, item_heights: Vec<u16>) {
        self.item_heights = item_heights;
        if self.item_heights.is_empty() {
            self.focused_index = 0;
            self.scroll_offset = 0;
            return;
        }
        self.focused_index = self.focused_index.min(self.item_count().saturating_sub(1));
        self.ensure_focused_visible();
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
