use super::{UiInputEvent, UiKeyInput};

#[derive(Clone, Debug, Default)]
pub struct TextInputState {
    value: String,
    cursor: usize,
    preferred_column: Option<usize>,
    soft_wrap_width: Option<usize>,
}

impl TextInputState {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self {
            value,
            cursor,
            preferred_column: None,
            soft_wrap_width: None,
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

    pub fn set_soft_wrap_width(&mut self, width: Option<usize>) {
        self.soft_wrap_width = width.map(|w| w.max(1));
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
                if let Some(width) = self.soft_wrap_width {
                    self.move_visual_vertical(-1, width);
                } else {
                    self.move_vertical(-1);
                }
                true
            }
            UiKeyInput::Down => {
                if let Some(width) = self.soft_wrap_width {
                    self.move_visual_vertical(1, width);
                } else {
                    self.move_vertical(1);
                }
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
        let total_lines = line_count(&self.value);
        if delta < 0 && line == 0 {
            self.cursor = 0;
            self.preferred_column = None;
            return;
        }
        if delta > 0 && line + 1 >= total_lines {
            let len = self.value.chars().count();
            self.cursor = len;
            self.preferred_column = None;
            return;
        }

        let target_line = (line as i32 + delta).clamp(0, total_lines as i32 - 1);
        let preferred = self.preferred_column.unwrap_or(col);
        self.cursor = cursor_for_line_col(&self.value, target_line as usize, preferred);
        self.preferred_column = Some(preferred);
    }

    fn move_visual_vertical(&mut self, delta: i32, wrap_width: usize) {
        let (row, col, total_rows) = visual_row_col_for_cursor(&self.value, self.cursor, wrap_width);
        if delta < 0 && row == 0 {
            self.cursor = 0;
            self.preferred_column = None;
            return;
        }
        if delta > 0 && row + 1 >= total_rows {
            let len = self.value.chars().count();
            self.cursor = len;
            self.preferred_column = None;
            return;
        }

        let target_row = (row as i32 + delta).clamp(0, total_rows as i32 - 1) as usize;
        let preferred = self.preferred_column.unwrap_or(col);
        self.cursor = cursor_for_visual_row_col(&self.value, wrap_width, target_row, preferred);
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

fn visual_row_col_for_cursor(value: &str, cursor: usize, wrap_width: usize) -> (usize, usize, usize) {
    let width = wrap_width.max(1);
    let chars: Vec<char> = value.chars().collect();
    let mut row = 0usize;
    let mut col = 0usize;

    for (i, ch) in chars.iter().copied().enumerate() {
        if ch != '\n' {
            let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if col > 0 && col.saturating_add(w) > width {
                row += 1;
                col = 0;
            }
        }

        if i == cursor {
            let total_rows = total_visual_rows(value, width);
            return (row, col, total_rows);
        }

        if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            col = col.saturating_add(w);
        }
    }

    let total_rows = total_visual_rows(value, width);
    (row, col, total_rows)
}

fn total_visual_rows(value: &str, wrap_width: usize) -> usize {
    let width = wrap_width.max(1);
    let mut rows = 1usize;
    let mut col = 0usize;
    for ch in value.chars() {
        if ch == '\n' {
            rows += 1;
            col = 0;
            continue;
        }
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if col > 0 && col.saturating_add(w) > width {
            rows += 1;
            col = 0;
        }
        col = col.saturating_add(w);
    }
    rows
}

fn cursor_for_visual_row_col(value: &str, wrap_width: usize, target_row: usize, target_col: usize) -> usize {
    let width = wrap_width.max(1);
    let chars: Vec<char> = value.chars().collect();
    let mut row = 0usize;
    let mut col = 0usize;
    let mut best = None;
    let mut first = None;
    let mut last = None;

    for (i, ch) in chars.iter().copied().enumerate() {
        if ch != '\n' {
            let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if col > 0 && col.saturating_add(w) > width {
                row += 1;
                col = 0;
            }
        }

        if row == target_row {
            first.get_or_insert(i);
            last = Some(i);
            if col <= target_col {
                best = Some(i);
            }
        } else if row > target_row {
            break;
        }

        if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            col = col.saturating_add(w);
        }
    }

    let end = chars.len();
    if row == target_row {
        first.get_or_insert(end);
        last = Some(end);
        if col <= target_col {
            best = Some(end);
        }
    }

    best.or(last).or(first).unwrap_or(end)
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
