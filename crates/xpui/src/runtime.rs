use crate::{FocusId, Node};

pub trait UiApp {
    fn render(&mut self) -> Node;

    fn on_input(&mut self, _event: UiInputEvent) {}

    fn set_window_size(&mut self, _size: WindowSize) {}

    fn focus_state(&mut self) -> Option<&mut FocusState> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiKeyInput {
    Tab,
    BackTab,
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Backspace,
    BackspaceWord,
    Delete,
    Enter,
    Esc,
    Char(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputEvent {
    Key(UiKeyInput),
    ScrollLines(i16),
}

#[derive(Clone, Debug, Default)]
pub struct TextInputState {
    value: String,
    cursor: usize,
    preferred_column: Option<usize>,
}

impl TextInputState {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self {
            value,
            cursor,
            preferred_column: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.cursor.min(self.value.chars().count());
        self.preferred_column = None;
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.value.chars().count());
        self.preferred_column = None;
    }

    pub fn handle_input(&mut self, event: UiInputEvent) -> bool {
        let UiInputEvent::Key(key) = event else {
            return false;
        };

        match key {
            UiKeyInput::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                self.preferred_column = None;
                true
            }
            UiKeyInput::Right => {
                self.cursor = (self.cursor + 1).min(self.value.chars().count());
                self.preferred_column = None;
                true
            }
            UiKeyInput::WordLeft => {
                self.cursor = prev_word_boundary(&self.value, self.cursor);
                true
            }
            UiKeyInput::WordRight => {
                self.cursor = next_word_boundary(&self.value, self.cursor);
                true
            }
            UiKeyInput::Home => {
                self.cursor = 0;
                self.preferred_column = None;
                true
            }
            UiKeyInput::End => {
                self.cursor = self.value.chars().count();
                self.preferred_column = None;
                true
            }
            UiKeyInput::Up => {
                self.move_vertical(-1);
                true
            }
            UiKeyInput::Down => {
                self.move_vertical(1);
                true
            }
            UiKeyInput::BackspaceWord => {
                if self.cursor == 0 {
                    return false;
                }
                let start_char = prev_word_boundary(&self.value, self.cursor);
                let start = char_to_byte_index(&self.value, start_char);
                let end = char_to_byte_index(&self.value, self.cursor);
                self.value.replace_range(start..end, "");
                self.cursor = start_char;
                self.preferred_column = None;
                true
            }
            UiKeyInput::Backspace => {
                if self.cursor == 0 {
                    return false;
                }
                let end = char_to_byte_index(&self.value, self.cursor);
                let start = char_to_byte_index(&self.value, self.cursor - 1);
                self.value.replace_range(start..end, "");
                self.cursor -= 1;
                self.preferred_column = None;
                true
            }
            UiKeyInput::Delete => {
                let len = self.value.chars().count();
                if self.cursor >= len {
                    return false;
                }
                let start = char_to_byte_index(&self.value, self.cursor);
                let end = char_to_byte_index(&self.value, self.cursor + 1);
                self.value.replace_range(start..end, "");
                self.preferred_column = None;
                true
            }
            UiKeyInput::Char(ch) => {
                let idx = char_to_byte_index(&self.value, self.cursor);
                self.value.insert(idx, ch);
                self.cursor += 1;
                self.preferred_column = None;
                true
            }
            UiKeyInput::Enter => {
                let idx = char_to_byte_index(&self.value, self.cursor);
                self.value.insert(idx, '\n');
                self.cursor += 1;
                self.preferred_column = None;
                true
            }
            _ => false,
        }
    }

    fn move_vertical(&mut self, delta: i32) {
        let (line, col) = line_col_for_cursor(&self.value, self.cursor);
        let target_line = (line as i32 + delta).clamp(0, line_count(&self.value) as i32 - 1);
        let preferred = self.preferred_column.unwrap_or(col);
        self.cursor = cursor_for_line_col(&self.value, target_line as usize, preferred);
        self.preferred_column = Some(preferred);
    }
}

fn char_to_byte_index(value: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }
    value
        .char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len())
}

fn line_count(value: &str) -> usize {
    value.chars().filter(|ch| *ch == '\n').count() + 1
}

fn line_col_for_cursor(value: &str, cursor: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;
    for (i, ch) in value.chars().enumerate() {
        if i == cursor {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn cursor_for_line_col(value: &str, target_line: usize, target_col: usize) -> usize {
    let mut line = 0usize;
    let mut col = 0usize;
    let mut idx = 0usize;

    for ch in value.chars() {
        if line == target_line && col >= target_col {
            break;
        }

        if ch == '\n' {
            if line == target_line {
                break;
            }
            line += 1;
            col = 0;
            idx += 1;
            continue;
        }

        if line == target_line {
            col += 1;
        }
        idx += 1;
    }

    idx
}

fn prev_word_boundary(value: &str, cursor: usize) -> usize {
    let chars: Vec<char> = value.chars().collect();
    let mut i = cursor.min(chars.len());

    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }

    i
}

fn next_word_boundary(value: &str, cursor: usize) -> usize {
    let chars: Vec<char> = value.chars().collect();
    let mut i = cursor.min(chars.len());

    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    while i < chars.len() && !chars[i].is_whitespace() {
        i += 1;
    }

    i
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
